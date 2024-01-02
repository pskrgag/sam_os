#![macro_use]

use rtl::vmm::types::*;

extern "C" {
    static load_addr: usize;
    static start: usize;
    static mmio_end: usize;
    static end: usize;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        #[allow(unused_unsafe)]
        unsafe {
            &$a as *const usize as usize
        }
    }};
}

#[inline]
pub fn image_size() -> usize {
    linker_var!(mmio_end) - linker_var!(start)
}

#[inline]
pub fn image_end_rounded() -> VirtAddr {
    unsafe { *VirtAddr::from_raw(&end as *const _).round_up_page() }
}
