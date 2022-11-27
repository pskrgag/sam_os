use crate::{
    arch,
    kernel::locking::spinlock::Spinlock,
    mm::{
        allocators::page_alloc::PAGE_ALLOC,
        paging::kernel_page_table::kernel_page_table,
        paging::page_table::{MappingType, PageTable},
        types::*,
        MemRange,
    },
};
use core::{
    mem::{size_of, transmute},
    ptr::NonNull,
};

#[repr(C)]
struct SlabBlockHeader {
    next: Option<NonNull<SlabBlockHeader>>,
}

#[repr(C)]
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

static KERNEL_SLABS: [Spinlock<SlabAllocator>; 6] = [
    Spinlock::new(SlabAllocator::default()),
    Spinlock::new(SlabAllocator::default()),
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
            let mut block: Option<*mut SlabBlockHeader> = None;

            *(header as *mut u8) = 0x10;

            (*header).free = NonNull::new(firts_free_block.to_raw_mut::<SlabBlockHeader>());
            (*header).link = None;

            va.add(arch::PAGE_SIZE);

            while (firts_free_block.get() + size_of::<SlabCacheHeader>()) < va.get() {
                block = Some(firts_free_block.to_raw_mut::<SlabBlockHeader>());
                firts_free_block.add(obj_size).round_up(obj_size);

                (*block.unwrap()).next =
                    NonNull::new(firts_free_block.to_raw_mut::<SlabBlockHeader>());
            }

            if block.is_some() {
                (*block.unwrap()).next = None;
            }

            println!("Kernel slab {} initialized...", obj_size);
            Some(header)
        }
    }

    /* Pointer to element and is_full */
    pub unsafe fn alloc(&mut self) -> Option<(*mut u8, bool)> {
        let mut first_free = self.free?;
        let first_free_ref = first_free.as_mut();

        self.free = first_free_ref.next;

        if self.free.is_some() {
            Some((first_free.as_ptr() as *mut u8, true))
        } else {
            Some((first_free.as_ptr() as *mut u8, false))
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

    pub unsafe fn alloc(&mut self) -> Option<*mut u8> {
        if self.partial.is_some() {
            let ptr = self.partial.unwrap().as_mut().alloc()?;

            Some(ptr.0)
        } else if self.free.is_some() {
            let ptr = self.free.unwrap().as_mut().alloc()?;

            Some(ptr.0)
        } else {
            let free_slab = NonNull::new(SlabCacheHeader::new(self.slab_size)?)?;

            self.free = Some(free_slab);
            Some(self.free.unwrap().as_mut().alloc()?.0)
        }
    }
}

unsafe impl Send for SlabAllocator {}

pub fn alloc(mut size: usize) -> Option<*mut u8> {
    if size < 4 {
        size = 4;
    }

    let slab_index = (size.next_power_of_two().ilog2() as usize) - 2;

    if slab_index >= KERNEL_SLABS.len() {
        return None;
    }

    unsafe { (*KERNEL_SLABS[slab_index].lock()).alloc() }
}

pub fn init_kernel_slabs() -> Option<()> {
    let mut size: usize = 4;

    for i in &KERNEL_SLABS {
        (*i.lock()) = SlabAllocator::new(size)?;
        size = (size + 1).next_power_of_two();
    }

    crate::mm::allocators::allocator::BOOT_ALLOC_IS_DEAD
        .store(false, core::sync::atomic::Ordering::Relaxed);

    Some(())
}
