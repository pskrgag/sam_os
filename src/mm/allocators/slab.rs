use crate::{
    arch,
    mm::{
        allocators::page_alloc::PAGE_ALLOC,
        paging::kernel_page_table::kernel_page_table,
        paging::page_table::{MappingType, PageTable},
        types::*,
        MemRange,
    },
    kernel::locking::spinlock::Spinlock,
};
use core::{
    mem::{size_of, transmute},
    ptr::NonNull,
};

#[repr(C)]
struct SlabBlockHeader {
    next: Option<NonNull<SlabBlockHeader>>,
}

#[repr(C, packed)]
struct SlabCacheHeader {
    free: Option<NonNull<SlabBlockHeader>>,
    link: Option<NonNull<SlabBlockHeader>>,
}

pub struct SlabAllocator {
    free: Option<NonNull<SlabCacheHeader>>,
    partial: Option<NonNull<SlabCacheHeader>>,
    occupied: Option<NonNull<SlabCacheHeader>>,
    slab_size: usize,
}


static mut KERNEL_SLABS: [Spinlock<SlabAllocator>; 4] = [
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
];


impl SlabCacheHeader {
    pub fn new(obj_size: usize) -> Option<*mut Self> {
        let page = PAGE_ALLOC.lock().alloc_pages(1)?;
        let mut page_va = VirtAddr::from(page);

        (*kernel_page_table())
            .map(
                None,
                MemRange::new(page_va, arch::PAGE_SIZE),
                MappingType::KernelData,
            )
            .ok()?;

        let mut va = page_va;
        let firts_free_block = page_va.add(size_of::<SlabCacheHeader>()).round_up(obj_size);

        unsafe {
            let header: *mut SlabCacheHeader = transmute::<_, _>(va.to_raw::<u8>());

            *(header as *mut u8) = 0x10;

            (*header).free = NonNull::new(firts_free_block.to_raw_mut::<SlabBlockHeader>());
            (*header).link = None;


            va.add(arch::PAGE_SIZE);

            while (firts_free_block.get() + size_of::<SlabCacheHeader>()) <= va.get() {
                let block = firts_free_block.to_raw_mut::<SlabBlockHeader>();
                firts_free_block.add(obj_size).round_up(obj_size);

                (*block).next = NonNull::new(firts_free_block.to_raw_mut::<SlabBlockHeader>());
            }

            Some(header)
        }
    }
}

impl SlabAllocator {
    pub fn new(obj_size: usize) -> Option<Self> {
        Some(Self {
            slab_size: obj_size,
            free: NonNull::new(SlabCacheHeader::new(obj_size)?),
            partial: None,
            occupied: None,
        })
    }

    pub const fn default() -> Self {
        Self {
            slab_size: 0,
            free: None,
            partial: None,
            occupied: None,
        }
    }
}

unsafe impl Send for SlabAllocator { }

pub fn init_kernel_slabs() -> Option<()> {
    let mut size: usize = 4;

    println!("Initing kernel slabs...");

    unsafe {
        for i in &KERNEL_SLABS {
            (*i.lock()) = SlabAllocator::new(size)?;
            size = size.next_power_of_two();
        }
    }

    Some(())
}
