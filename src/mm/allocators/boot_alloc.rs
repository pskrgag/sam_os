use crate::{arch::PAGE_SIZE, kernel::locking::fake_lock::FakeLock};
use core::{
    alloc::Layout,
    mem::{size_of, size_of_val, transmute},
    ptr::NonNull,
};

#[repr(C)]
struct FfHeader {
    next: Option<NonNull<FfHeader>>,
    prev: Option<NonNull<FfHeader>>,
    size: usize,
    free: bool,
}

pub struct BootAlloc {
    pool: [u8; PAGE_SIZE * 40],
    free: usize,
}

pub static BOOT_ALLOC: FakeLock<BootAlloc> = FakeLock::new(BootAlloc::default());

unsafe impl Sync for BootAlloc {}

impl BootAlloc {
    pub(self) const fn default() -> Self {
        Self {
            pool: [0; PAGE_SIZE * 40],
            free: core::mem::size_of::<Self>() - core::mem::size_of::<FfHeader>(),
        }
    }

    fn update_header_next(header: &mut NonNull<FfHeader>, next: Option<NonNull<FfHeader>>) {
        unsafe { header.as_mut().next = next };
    }

    fn update_header_prev(header: &mut NonNull<FfHeader>, prev: Option<NonNull<FfHeader>>) {
        unsafe { header.as_mut().prev = prev };
    }

    fn update_header_size(header: &mut NonNull<FfHeader>, size: usize) {
        unsafe { header.as_mut().size = size };
    }

    fn update_header_free(header: &mut NonNull<FfHeader>, free: bool) {
        unsafe { header.as_mut().free = free };
    }

    pub fn init(&mut self) {
        let mut tmp_head: NonNull<FfHeader> =
            unsafe { NonNull::new(transmute::<_, _>(&self.pool)).unwrap() };

        Self::update_header_next(&mut tmp_head, None);
        Self::update_header_prev(&mut tmp_head, None);
        Self::update_header_free(&mut tmp_head, true);
        Self::update_header_size(
            &mut tmp_head,
            size_of_val(&self.pool) - size_of::<FfHeader>(),
        );

        println!(
            "Boot allocator initialized: pool size {}",
            size_of_val(&self.pool)
        );
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let mut iter;
        let size = layout.size();

        iter = NonNull::new(unsafe {
            core::mem::transmute::<_, *mut FfHeader>(self.pool.as_mut_ptr())
        });

        while iter.is_some() {
            let header = iter.unwrap().as_mut() as *mut FfHeader;
            let header_raw = header as *mut u8;

            if (*header).size > size && (*header).free {
                let next_block = core::mem::transmute::<_, *mut FfHeader>(
                    header_raw.offset((core::mem::size_of::<FfHeader>() + size) as isize),
                );
                let new_block = FfHeader {
                    prev: NonNull::new(header),
                    next: (*header).next,
                    size: (*header).size - size - core::mem::size_of::<FfHeader>(),
                    free: true,
                };

                if new_block.next.is_some() {
                    new_block.next.unwrap().as_mut().prev = NonNull::new(next_block as *mut _);
                }

                *next_block = new_block;

                (*header).next = NonNull::new(next_block);
                (*header).size = size;
                (*header).free = false;

                //        println!("Boot alloc {:p} {:p}", &self.pool, header_raw.offset(core::mem::size_of::<FfHeader>() as isize));
                return header_raw.offset(core::mem::size_of::<FfHeader>() as isize);
            }

            iter = (*header).next;
        }

        panic!();
    }

    pub unsafe fn free(&mut self, data: *mut u8) {
        let mut cur_header = unsafe {
            core::mem::transmute::<_, *mut FfHeader>(
                data.offset(-(core::mem::size_of::<FfHeader>() as isize)),
            )
            .as_mut()
            .unwrap()
        };

        let prev = cur_header.prev;
        let next = cur_header.prev;

        if next.is_some()
            && next.unwrap().as_mut().free
            && prev.is_some()
            && prev.unwrap().as_mut().free
        {
            prev.unwrap().as_mut().size = cur_header.size
                + core::mem::size_of::<FfHeader>() * 2
                + next.unwrap().as_ref().size;

            if next.unwrap().as_mut().next.is_some() {
                next.unwrap().as_mut().next.unwrap().as_mut().prev = cur_header.prev;
            }

            prev.unwrap().as_mut().next = next.unwrap().as_ref().next;
        } else if prev.is_some() && prev.unwrap().as_ref().free {
            prev.unwrap().as_mut().size += cur_header.size + core::mem::size_of::<FfHeader>();
            prev.unwrap().as_mut().next = next;

            if next.is_some() {
                next.unwrap().as_mut().prev = prev;
            }
        } else if next.is_some() && next.unwrap().as_ref().free {
            cur_header.size += next.unwrap().as_ref().size + core::mem::size_of::<FfHeader>();
            cur_header.next = next.unwrap().as_ref().next;
            next.unwrap().as_mut().prev = NonNull::new(cur_header as *mut _);
        }
    }
}

pub fn init() {
    BOOT_ALLOC.get().init();
}
