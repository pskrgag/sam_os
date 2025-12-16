use crate::arch::backtrace::backtrace;
use crate::drivers::irq::irq::irq_dispatch;
use crate::arch::regs::{Context, TrapReason};
use core::arch::global_asm;
use rtl::linker_var;
use hal::address::*;
use aarch64_cpu::registers::{VBAR_EL1, Writeable, ESR_EL1, Readable, FAR_EL1, ELR_EL1};

global_asm!(include_str!("interrupts.s"));

unsafe extern "C" {
    static exception_vector: usize;
    static sfixup: usize;
    static efixup: usize;
}

#[repr(C)]
#[derive(Debug)]
struct FixupEntry {
    pub fault: VirtAddr,
    pub fix: VirtAddr,
}

#[inline]
pub fn set_up_vbar() {
    VBAR_EL1.set(linker_var!(exception_vector) as u64);
}

fn fixup(v: VirtAddr, ctx: &mut Context) -> bool {
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

fn kern_sync(ctx: &mut Context) {
    let elr_el1: VirtAddr = ctx.elr.into();
    let esr_el1 = ESR_EL1.get();
    let far_el1 = FAR_EL1.get();

    if !fixup(elr_el1, ctx) {
        println!("!!! Kernel sync exception");
        println!("{:?}", ctx);
        println!(
            "ESR_EL1 0x{:x} FAR_EL1 0x{:x}, ELR_EL1 0x{:x}",
            esr_el1, far_el1, elr_el1
        );

        let mut bt = [VirtAddr::new(0); 50];

        unsafe { backtrace(&mut bt, ctx.x29.into()) };

        println!("--- cut here ---");
        println!("Kernel backtrace");

        for (i, addr) in bt.iter().take_while(|x| !x.is_null()).enumerate() {
            println!("#{} [{:p}]", i, addr.to_raw::<usize>());
        }

        loop {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(ctx: &mut Context) {
    match ctx.trap_reason() {
        TrapReason::Irq => irq_dispatch(),
        TrapReason::PageFault => kern_sync(ctx),
        _ => kern_exception_bug(),
    }
}

pub fn kern_exception_bug() -> ! {
    let esr_el1 = ESR_EL1.get();
    let far_el1 = FAR_EL1.get();
    let elr_el1 = ELR_EL1.get();

    println!("Something weird happened");
    println!(
        "ESR_EL1 0x{:x} FAR_EL1 0x{:x}, ELR_EL1 0x{:x}",
        esr_el1, far_el1, elr_el1
    );
    println!("No idea how to deal with 0x{:x}", esr_el1);

    panic!();
}
