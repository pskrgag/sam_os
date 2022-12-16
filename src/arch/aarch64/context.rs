use core::arch::global_asm;

global_asm!(
    ".globl switch_to",
    "switch_to:",
    // Safe current ctx
    "stp        x19, x20, [x0, #0]",
    "stp        x21, x22, [x0, #0x10]",
    "stp        x23, x24, [x0, #0x20]",
    "stp        x25, x26, [x0, #0x30]",
    "stp        x27, x28, [x0, #0x40]",
    "stp        x29, x30, [x0, #0x50]",
    "mov        x4, sp",
    "mov        x5, fp",
    "stp        x4, x5, [x0, #0x60]",
    // Restore ctx
    "ldp        x19, x20, [x1, #0]",
    "ldp        x21, x22, [x1, #0x10]",
    "ldp        x23, x24, [x1, #0x20]",
    "ldp        x25, x26, [x1, #0x30]",
    "ldp        x27, x28, [x1, #0x40]",
    "ldp        x29, x30, [x1, #0x50]",
    "ldp        x4, x5, [x1, #0x60]",
    "mov        sp, x4",
    "mov        fp, x5",
    "ret",
);
