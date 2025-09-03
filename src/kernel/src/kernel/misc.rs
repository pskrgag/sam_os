#![macro_use]

use rtl::vmm::types::*;

unsafe extern "C" {
    static load_addr: usize;
    static __start: usize;
    static mmio_end: usize;
    static __end: usize;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        #[allow(unused_unsafe)]
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            &$a as *const usize as usize
        }
    }};
}

#[inline]
pub fn image_size() -> usize {
    linker_var!(__end) - linker_var!(__start)
}

#[inline]
pub fn image_end_rounded() -> VirtAddr {
    unsafe { *VirtAddr::from_raw(&__end as *const _).round_up_page() }
}
