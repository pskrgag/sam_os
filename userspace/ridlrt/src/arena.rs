use core::mem;

#[derive(Debug)]
pub struct MessageArena<'a> {
    free: &'a [u8],
    start: usize,
    allocated: usize,
    size: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct ArenaPtr<T> {
    pub offset: usize,
    pub size: usize,
    pub _p: core::marker::PhantomData<T>,
}

impl<'a> MessageArena<'a> {
    pub fn new_backed(free: &'a mut [u8]) -> Self {
        Self {
            free,
            start: free.as_ptr() as usize,
            allocated: 0,
            size: free.len(),
        }
    }

    pub fn allocate<T>(&mut self) -> Option<ArenaPtr<T>> {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        let diff =
            (self.free.as_ptr() as usize - 1).next_multiple_of(align) - self.free.as_ptr() as usize;

        let alloc = self.free.get(diff..diff + size)?;

        self.free = &self.free[diff + size..];
        self.allocated = self.free.as_ptr() as usize - self.start;

        Some(ArenaPtr {
            offset: alloc.as_ptr() as usize - self.start,
            size,
            _p: core::marker::PhantomData,
        })
    }

    pub fn store<T: Copy>(&mut self, ptr: ArenaPtr<T>, data: &T) {
        // TODO: sanity cheks
        let s = (self.start as usize + ptr.offset) as *mut T;

        unsafe {
            *s = *data;
        }
    }

    pub fn read<T: Copy>(&mut self, ptr: ArenaPtr<T>) -> Option<T> {
        // TODO: sanity cheks
        let s = (self.start as usize + ptr.offset) as *mut T;

        unsafe {
            Some(*s)
        }
    }

    pub fn as_slice_allocated(&self) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(self.start as *const _, self.allocated) }
    }

    pub fn as_slice(&self) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(self.start as *const _, self.size) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_alloc() {
        let mut buff = [0u8; 100];
        let mut arena = MessageArena::new_backed(buff.as_mut_slice());

        let p = arena.allocate::<i32>();
        assert!(p.is_some());

        let p = p.unwrap();
        assert_eq!(p.offset, 0);
        assert_eq!(p.size, core::mem::size_of::<i32>());
    }

    #[test]
    fn basic_alloc_align() {
        #[repr(align(128))]
        struct Aligned {
            _i: i32,
        }

        let mut buff = [0u8; 1000];
        let mut arena = MessageArena::new_backed(buff.as_mut_slice());

        let p = arena.allocate::<Aligned>();
        assert!(p.is_some());

        let p = p.unwrap();
        assert_eq!(p.size, 128);
        assert_eq!(
            p.offset,
            (buff.as_ptr() as usize).next_multiple_of(128) - buff.as_ptr() as usize
        );
    }

    #[test]
    fn basic_alloc_to_do_intersect() {
        let mut buff = [0u8; 1000];
        let mut arena = MessageArena::new_backed(buff.as_mut_slice());

        let p = arena.allocate::<u64>();
        let p1 = arena.allocate::<u64>();
        assert!(p.is_some());
        assert!(p1.is_some());

        let p = p.unwrap();
        let p1 = p1.unwrap();

        assert!(p.offset + core::mem::size_of::<u64>() <= p1.offset);
    }
}
