use crate::arch::PAGE_SIZE;
use crate::vmm::alloc::BackendAllocator;
use crate::vmm::types::*;

pub struct SlabAllocator<B: BackendAllocator + 'static> {
    slab_size: usize,
    freelist: FreeList,
    backend: Option<&'static B>,
}

struct FreeList {
    next: Option<&'static mut FreeList>,
}

impl<B: BackendAllocator> SlabAllocator<B> {
    pub const fn default() -> Self {
        Self {
            slab_size: 0,
            freelist: FreeList::default(),
            backend: None,
        }
    }

    pub fn new(size: usize, backend: &'static B) -> Option<Self> {
        Some(Self {
            slab_size: size,
            freelist: FreeList::new(size, backend)?,
            backend: Some(backend),
        })
    }

    pub fn alloc(&mut self) -> Option<*mut u8> {
        match self
            .freelist
            .alloc()
            .map(|ptr| ptr as *mut FreeList as *mut u8)
        {
            Some(ptr) => Some(ptr),
            None => {
                let new_list = FreeList::new(self.slab_size, self.backend.unwrap())?;
                self.freelist.add_to_freelist(new_list.next.unwrap());

                self.freelist
                    .alloc()
                    .map(|ptr: &mut FreeList| ptr as *mut FreeList as *mut u8)
            }
        }
    }

    pub fn free(&mut self, addr: *mut u8) {
        self.freelist
            .add_to_freelist(unsafe { &mut *(addr as *mut FreeList) });
    }
}

impl FreeList {
    /* Allocate one page for the beggining */
    pub fn new<B: BackendAllocator>(size: usize, backend: &B) -> Option<Self> {
        assert!(size.is_power_of_two());

        let mut va = VirtAddr::from_raw(backend.allocate(1)?);
        let block_count = PAGE_SIZE / size;
        let mut list = Self::default();

        for _ in 0..block_count {
            let new = va.to_raw_mut::<Self>();
            list.add_to_freelist(unsafe { &mut *new });
            va.add(size);
        }

        Some(list)
    }

    pub fn add_to_freelist(&mut self, new: &'static mut Self) {
        match self.next.take() {
            Some(l) => {
                new.next = Some(l);
                self.next = Some(new);
            }
            None => self.next = Some(new),
        }
    }

    pub fn alloc(&mut self) -> Option<&mut Self> {
        let next = self.next.take()?;

        self.next = next.next.take();
        Some(next)
    }

    pub const fn default() -> Self {
        Self { next: None }
    }
}
