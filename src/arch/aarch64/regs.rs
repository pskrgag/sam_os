#[repr(C, packed)]
pub struct Context {
    sp: usize,
    lr: usize,
    fp: usize,
    // Callee-saved registers
    x19: usize,
    x20: usize,
    x21: usize,
    x22: usize,
    x23: usize,
    x24: usize,
    x25: usize,
    x26: usize,
    x27: usize,
    x28: usize,
    x29: usize,
}
