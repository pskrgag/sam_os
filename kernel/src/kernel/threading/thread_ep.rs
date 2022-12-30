use core::arch::global_asm;

global_asm!(
    ".globl kernel_thread_entry_point",
    "kernel_thread_entry_point:",
    "mov   x0, x19",
    "br    x20",
);

global_asm!(
    ".globl user_thread_entry_point",
    "user_thread_entry_point:",
    "msr    ELR_EL1, x20",
    "mov    x0, sp",
    "msr    sp_el0, x0",
    "msr    spsr_el1, xzr",
    "eret",
);

pub fn idle_thread(_: ()) -> Option<()> {
    loop {
        println!("Idle loop");
        for _ in 0..10_000_000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}

pub fn idle_thread1(_: ()) -> Option<()> {
    loop {
        println!("Idle loop 1");
        for _ in 0..10_000_000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}