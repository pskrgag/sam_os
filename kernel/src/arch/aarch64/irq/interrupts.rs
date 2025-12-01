use aarch64_cpu::registers::{Readable, Writeable, DAIF};

#[derive(Copy, Clone)]
pub struct IrqFlags(usize);

#[inline]
pub fn get_flags() -> IrqFlags {
    IrqFlags(DAIF.get() as usize)
}

#[inline]
pub unsafe fn set_flags(flags: IrqFlags) {
    DAIF.set(flags.0 as u64)
}
