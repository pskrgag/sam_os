use crate::mm::allocators::page_alloc::page_allocator;
use hal::{
    address::{LinearAddr, PhysAddr, VirtAddr},
    arch::PAGE_SIZE,
};
use loader_protocol::LoaderArg;
use spin::Once;

pub struct Fdt {
    pub base: LinearAddr,
    pub size: usize,
}

pub static FDT: Once<Fdt> = Once::new();

pub fn init(arg: &LoaderArg) {
    let total_pages = arg.fdt_size.next_multiple_of(PAGE_SIZE) / PAGE_SIZE;
    let pages = page_allocator()
        .alloc(total_pages)
        .expect("Failed to allocate memory for FDT");

    let va_from: VirtAddr = LinearAddr::from(PhysAddr::new(arg.fdt_base)).into();
    let linear: LinearAddr = pages.into();
    let mut va_to: VirtAddr = linear.into();

    unsafe {
        va_to
            .as_slice_mut::<u8>(arg.fdt_size)
            .copy_from_slice(va_from.as_slice::<u8>(arg.fdt_size));
    }

    FDT.call_once(|| Fdt {
        base: linear,
        size: arg.fdt_size,
    });
}

pub fn fdt() -> &'static Fdt {
    unsafe { FDT.get_unchecked() }
}
