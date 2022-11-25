#![macro_use]

extern "C" {
    static load_addr: usize;
    static start: usize;
    static end: usize;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        unsafe { &$a as *const usize as usize }
    }};
}

pub const _1GB: usize = 1 << 30;
pub const _2MB: usize = 2 << 20;
pub const _4KB: usize = 1 << 12;

pub fn kernel_offset() -> usize {
    linker_var!(start) - linker_var!(load_addr)
}

#[inline]
pub fn image_size() -> usize {
    linker_var!(end) - linker_var!(start)
}
