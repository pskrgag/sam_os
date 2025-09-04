use crate::drivers::irq::irq::irq_dispatch;
use crate::kernel::sched;
use crate::kernel::syscalls::do_syscall;
use crate::linker_var;
use core::arch::{asm, global_asm};
use rtl::vmm::types::*;

global_asm!(include_str!("interrupts.S"));

unsafe extern "C" {
    static exception_vector: u64;
    static sfixup: usize;
    static efixup: usize;
}

#[repr(C)]
#[derive(Debug)]
struct FixupEntry {
    pub fault: VirtAddr,
    pub fix: VirtAddr,
}

#[repr(C)]
#[derive(Debug)]
pub struct ExceptionCtx {
    pub x0: usize,
    pub x1: usize,
    pub x2: usize,
    pub x3: usize,
    pub x4: usize,
    pub x5: usize,
    pub x6: usize,
    pub x7: usize,
    pub x8: usize,
    pub x9: usize,
    pub x10: usize,
    pub x11: usize,
    pub x12: usize,
    pub x13: usize,
    pub x14: usize,
    pub x15: usize,
    pub x16: usize,
    pub x17: usize,
    pub x18: usize,
    pub x19: usize,
    pub x20: usize,
    pub x21: usize,
    pub x22: usize,
    pub x23: usize,
    pub x24: usize,
    pub x25: usize,
    pub x26: usize,
    pub x27: usize,
    pub x28: usize,
    pub x29: usize,
    pub elr: usize,
    pub spsr: usize,
    pub sp_el0: usize,
    pub x30: usize,
}

impl ExceptionCtx {
    pub fn fp(&self) -> usize {
        self.x29
    }
}

#[inline]
pub fn set_up_vbar() {
    unsafe {
        asm!("msr VBAR_EL1, {}",
             "isb",
            in(reg) &exception_vector);
    }
}

fn fixup(v: VirtAddr, ctx: &mut ExceptionCtx) -> bool {
    let size = (linker_var!(efixup) - linker_var!(sfixup)) / core::mem::size_of::<FixupEntry>();
    let array =
        unsafe { core::slice::from_raw_parts(linker_var!(sfixup) as *const FixupEntry, size) };
    let mut found = false;

    for i in array {
        if i.fault == v {
            ctx.elr = i.fix.into();
            found = true;
            break;
        }
    }

    found
}

#[unsafe(no_mangle)]
pub extern "C" fn kern_sync64(
    esr_el1: VirtAddr,
    far_el1: VirtAddr,
    elr_el1: VirtAddr,
    ctx: &mut ExceptionCtx,
) {
    if !fixup(elr_el1, ctx) {
        println!("!!! Kernel sync exception");
        println!("{:?}", ctx);
        println!(
            "ESR_EL1 0x{:x} FAR_EL1 0x{:x}, ELR_EL1 0x{:x}",
            esr_el1, far_el1, elr_el1
        );

        panic!("Unhandled kernel sync exception");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kern_irq() {
    irq_dispatch();

    sched::run();
}

#[unsafe(no_mangle)]
pub extern "C" fn kern_exception_bug(esr_el1: usize, far_el1: usize, elr_el1: usize) -> ! {
    println!("Something weird happened");
    println!(
        "ESR_EL1 0x{:x} FAR_EL1 0x{:x}, ELR_EL1 0x{:x}",
        esr_el1, far_el1, elr_el1
    );
    println!("No idea how to deal with 0x{:x}", esr_el1);

    panic!();
}

#[unsafe(no_mangle)]
pub extern "C" fn user_sync(_ctx: &ExceptionCtx, esr_el1: usize, elr_el1: usize, far_el1: usize) {
    println!(
        "!!! Kernel sync from EL0    ESR_EL1 0x{:x}    ELR_EL1 0x{:x}    FAR_EL1 0x{:x}",
        esr_el1, elr_el1, far_el1,
    );

    // TODO: stop threads and kill task
    panic!("Some user thread has panicked! No idea how to deal with it");
}

#[unsafe(no_mangle)]
pub extern "C" fn user_syscall(ctx: &mut ExceptionCtx) {
    use crate::kernel::syscalls::SyscallArgs;

    if let Some(a) = SyscallArgs::new(
        ctx.x0,
        [ctx.x1, ctx.x2, ctx.x3, ctx.x4, ctx.x5, ctx.x6, ctx.x7],
    ) {
        let num = a.number();

        match do_syscall(a) {
            Ok(s) => ctx.x0 = s,
            Err(err) => {
                println!("err {:?} {}", err, num.bits());
                ctx.x0 = -(err as isize) as usize;
            }
        };
    } else {
        use rtl::error::ErrorType;
        ctx.x0 = (-(ErrorType::NoOperation as isize)) as usize;
    }
}
