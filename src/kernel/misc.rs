#![macro_use]

extern "C" {
    static load_addr: u64;
    static start: u64;
    static end: u64;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        unsafe { &$a as *const u64 as u64 }
    }};
}

pub fn kernel_offset() -> u64 {
    linker_var!(start) - linker_var!(load_addr)
}

#[inline]
pub fn image_size() -> u64 {
    linker_var!(end) - linker_var!(start)
}
