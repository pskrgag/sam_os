use crate::arch::backtrace::backtrace;
use crate::arch::regs::{Context, TrapReason};
use crate::drivers::irq::irq::irq_dispatch;
use aarch64_cpu::registers::{ELR_EL1, ESR_EL1, FAR_EL1, Readable, VBAR_EL1, Writeable};
use core::arch::global_asm;
use hal::address::*;
use rtl::linker_var;

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
        error!("!!! Kernel sync exception\n");
        error!("{ctx:?}\n");
        error!("ESR_EL1 0x{esr_el1:x} FAR_EL1 0x{far_el1:x}, ELR_EL1 0x{elr_el1:x}\n",);

        let mut bt = [VirtAddr::from_bits(0); 50];

        unsafe { backtrace(&mut bt, ctx.x29.into()) };

        error!("--- cut here ---\n");
        error!("Kernel backtrace\n");

        for (i, addr) in bt.iter().take_while(|x| !x.is_null()).enumerate() {
            error!("#{i} [{:p}]\n", addr.to_raw::<usize>());
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

    error!("Something weird happened");
    error!("ESR_EL1 0x{esr_el1:x} FAR_EL1 0x{far_el1:x}, ELR_EL1 0x{elr_el1:x}",);
    error!("No idea how to deal with 0x{esr_el1:x}");

    panic!();
}
