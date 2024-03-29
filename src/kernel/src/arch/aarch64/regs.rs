use core::mem::zeroed;

#[repr(C)]
pub struct Context {
    // Callee-saved registers
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
    pub lr: usize, // x30
    pub sp: usize,
    pub fp: usize,
    pub ttbr0: usize,
}

impl Default for Context {
    fn default() -> Self {
        let new: Self = unsafe { zeroed() };
        new
    }
}
