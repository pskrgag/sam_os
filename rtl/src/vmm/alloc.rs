pub trait BackendAllocator {
    fn allocate(&self, num_pages: usize) -> Option<*mut u8>;
    fn free(&self, p: *const u8, num_pages: usize);
}
