use crate::{arch::PAGE_SIZE, kernel::locking::fake_lock::FakeLock};
use core::{
    alloc::Layout,
    mem::{size_of, size_of_val, transmute},
    ptr::NonNull,
};

#[repr(C)]
struct FfHeader {
    next: Option<NonNull<FfHeader>>,
    size: usize,
    off: usize,
}

pub struct BootAlloc {
    pool: [u8; PAGE_SIZE * 10],
    head: Option<NonNull<FfHeader>>,
}

pub static BOOT_ALLOC: FakeLock<BootAlloc> = FakeLock::new(BootAlloc::default());

unsafe impl Sync for BootAlloc {}

impl BootAlloc {
    pub(self) const fn default() -> Self {
        Self {
            pool: [0; PAGE_SIZE * 10],
            head: None,
        }
    }

    fn update_header_next(header: &mut NonNull<FfHeader>, next: Option<NonNull<FfHeader>>) {
        unsafe { header.as_mut().next = next };
    }

    fn update_header_size(header: &mut NonNull<FfHeader>, size: usize) {
        unsafe { header.as_mut().size = size };
    }

    fn update_header_off(header: &mut NonNull<FfHeader>, off: usize) {
        unsafe { header.as_mut().off = off };
    }

    pub fn init(&mut self) {
        let mut tmp_head: NonNull<FfHeader> =
            unsafe { NonNull::new(transmute::<_, _>(&self.pool)).unwrap() };

        Self::update_header_next(&mut tmp_head, None);
        Self::update_header_size(
            &mut tmp_head,
            size_of_val(&self.pool) - size_of::<FfHeader>(),
        );
        Self::update_header_off(&mut tmp_head, 0);

        self.head = Some(tmp_head);

        println!(
            "Boot allocator initialized: pool size {}",
            size_of_val(&self.pool)
        );
    }

    pub(self) fn head(&mut self) -> Option<NonNull<FfHeader>> {
        self.head
    }

    pub(self) fn handle_alloc(
        &mut self,
        block: &mut FfHeader,
        prev: Option<&mut FfHeader>,
    ) -> *mut u8 {
        if block.next.is_none() && prev.is_none() {
            let start_ptr = unsafe { self.head.unwrap().as_ptr() };

            unsafe { start_ptr.offset(block.off as isize) as _ }
        } else {
            0x0 as _
        }
    }
}

pub fn alloc(layout: Layout) -> *mut u8 {
    let mut a = BOOT_ALLOC.get();
    let mut curr = unsafe { a.head().unwrap().as_mut() };
    let mut prev: Option<&mut FfHeader> = None;

    while {
        if curr.size >= layout.size() {}

        curr.next.is_some()
    } {}

    panic!("OOM in boot allocator; consider increasing builtin pool size")
}

pub fn free(layout: Layout) {}
