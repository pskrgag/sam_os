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
    "msr    SPSel, #1",
    "mov    sp, x19",
    "msr    sp_el0, x21",
    "msr    spsr_el1, xzr",
    "mov    x0, x23",
    "mov    x1, x24",
    "mov    x2, x25",
    "mov    x3, x26",
    "eor    x30, x30, x30",
    "eret",
);
