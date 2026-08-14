//! Fixed-capacity numeric ID allocator.

use rtl::error::ErrorType;

pub struct IdAllocator<const N: usize> {
    allocated: [bool; N],
    num_free: usize,
    next: usize,
}

impl<const N: usize> IdAllocator<N> {
    pub const fn new() -> Self {
        Self {
            allocated: [false; N],
            num_free: N,
            next: 0,
        }
    }

    pub fn num_free(&self) -> usize {
        self.num_free
    }

    pub fn allocate(&mut self) -> Option<usize> {
        if self.num_free == 0 {
            return None;
        }

        for offset in 0..N {
            let id = (self.next + offset) % N;

            if !self.allocated[id] {
                self.allocated[id] = true;
                self.num_free -= 1;
                self.next = (id + 1) % N;
                return Some(id);
            }
        }

        panic!("ID allocator free count is inconsistent")
    }

    pub fn allocate_specific(&mut self, id: usize) -> Result<usize, ErrorType> {
        if id >= N || self.allocated[id] {
            return Err(ErrorType::InvalidArgument);
        }

        self.allocated[id] = true;
        self.num_free -= 1;
        self.next = if N == 0 { 0 } else { (id + 1) % N };
        Ok(id)
    }

    pub fn free(&mut self, id: usize) {
        assert!(id < N);
        assert!(self.allocated[id]);

        self.allocated[id] = false;
        self.num_free += 1;
        self.next = id;
    }
}

impl<const N: usize> Default for IdAllocator<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::IdAllocator;

    #[test]
    fn allocates_all_ids() {
        let mut alloc = IdAllocator::<3>::new();

        assert_eq!(alloc.allocate(), Some(0));
        assert_eq!(alloc.allocate(), Some(1));
        assert_eq!(alloc.allocate(), Some(2));
        assert_eq!(alloc.allocate(), None);
        assert_eq!(alloc.num_free(), 0);
    }

    #[test]
    fn reuses_freed_id() {
        let mut alloc = IdAllocator::<3>::new();

        assert_eq!(alloc.allocate(), Some(0));
        assert_eq!(alloc.allocate(), Some(1));
        alloc.free(0);

        assert_eq!(alloc.allocate(), Some(0));
        assert_eq!(alloc.num_free(), 1);
    }

    #[test]
    fn reserves_specific_id() {
        let mut alloc = IdAllocator::<4>::new();

        assert_eq!(alloc.allocate_specific(2).unwrap(), 2);
        assert!(alloc.allocate_specific(2).is_err());
        assert!(alloc.allocate_specific(4).is_err());
        assert_eq!(alloc.num_free(), 3);
    }

    #[test]
    fn supports_empty_pool() {
        let mut alloc = IdAllocator::<0>::new();

        assert_eq!(alloc.allocate(), None);
        assert!(alloc.allocate_specific(0).is_err());
        assert_eq!(alloc.num_free(), 0);
    }
}
