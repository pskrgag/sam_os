use core::mem;

#[derive(Debug)]
pub struct MessageArena<'a> {
    free: &'a [u8],
    start: usize,
    allocated: usize,
    size: usize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
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

    fn allocate_impl<T>(&mut self, size: usize, align: usize) -> Option<ArenaPtr<T>> {
        let diff =
            (self.free.as_ptr() as usize).next_multiple_of(align) - self.free.as_ptr() as usize;

        let alloc = self.free.get(diff..diff + size)?;

        self.free = &self.free[diff + size..];
        self.allocated = self.free.as_ptr() as usize - self.start;

        Some(ArenaPtr {
            offset: alloc.as_ptr() as usize - self.start,
            size,
            _p: core::marker::PhantomData,
        })
    }

    pub fn allocate<T: Copy>(&mut self, t: &T) -> Option<ArenaPtr<T>> {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        let p = self.allocate_impl(size, align)?;
        self.store_impl(p, t as *const T, size);

        Some(p)
    }

    pub fn allocate_slice<T: Copy>(&mut self, t: &[T]) -> Option<ArenaPtr<T>> {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        let p = self.allocate_impl(size * t.len(), align)?;
        self.store_impl(p, t.as_ptr(), size * t.len());

        Some(p)
    }

    pub fn store_impl<T: Copy>(&mut self, ptr: ArenaPtr<T>, source: *const T, size: usize) {
        // TODO: sanity cheks
        let s = (self.start as usize + ptr.offset) as *mut T;

        unsafe {
            let dst = core::slice::from_raw_parts_mut(s, ptr.size / core::mem::size_of::<T>());
            let src = core::slice::from_raw_parts(source, size / core::mem::size_of::<T>());

            dst.copy_from_slice(src);
        }
    }

    pub fn read<T: Copy>(&mut self, ptr: ArenaPtr<T>) -> Option<T> {
        // TODO: sanity cheks
        let s = (self.start as usize + ptr.offset) as *mut T;

        unsafe { Some(*s) }
    }

    pub fn read_slice<T: Copy>(&mut self, ptr: ArenaPtr<T>, to: &mut [T]) -> Result<usize, ()> {
        // TODO: sanity cheks
        let s = (self.start as usize + ptr.offset) as *mut T;
        let count = ptr.size / core::mem::size_of::<T>();

        unsafe {
            let dst = core::slice::from_raw_parts_mut(s, count);
            to[..dst.len()].copy_from_slice(dst);
            Ok(dst.len())
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

        let p = arena.allocate::<i32>(&10);
        assert!(p.is_some());

        let p = p.unwrap();
        assert_eq!(p.offset, 0);
        assert_eq!(p.size, core::mem::size_of::<i32>());
    }

    #[test]
    fn basic_alloc_align() {
        #[repr(align(128))]
        #[derive(Copy, Clone)]
        struct Aligned {
            _i: i32,
        }

        let mut buff = [0u8; 1000];
        let mut arena = MessageArena::new_backed(buff.as_mut_slice());

        let p = arena.allocate::<Aligned>(&Aligned { _i: 10 });
        assert!(p.is_some());

        let p = p.unwrap();
        assert_eq!(p.size, 128);
        assert_eq!(
            p.offset,
            (buff.as_ptr() as usize).next_multiple_of(128) - buff.as_ptr() as usize
        );
    }

    #[test]
    fn basic_alloc_two_do_intersect() {
        let mut buff = [0u8; 1000];
        let mut arena = MessageArena::new_backed(buff.as_mut_slice());

        let p = arena.allocate::<u64>(&10);
        let p1 = arena.allocate::<u64>(&10);

        assert!(p.is_some());
        assert!(p1.is_some());

        let p = p.unwrap();
        let p1 = p1.unwrap();

        assert!(p.offset + core::mem::size_of::<u64>() <= p1.offset);
    }

    #[test]
    fn basic_alloc_and_read() {
        let mut buff = [0u8; 1000];
        let mut arena = MessageArena::new_backed(buff.as_mut_slice());

        let p = arena.allocate::<u64>(&10).unwrap();
        let p = arena.read(p).unwrap();

        assert_eq!(p, 10);
    }

    #[test]
    fn basic_alloc_and_read_slice() {
        let mut buff = [0u8; 1000];
        let mut arena = MessageArena::new_backed(buff.as_mut_slice());

        let mut slice = [0u8; 100];
        let p = arena.allocate_slice("hello".as_bytes()).unwrap();
        let size = arena.read_slice(p, &mut slice).unwrap();

        assert_eq!(
            core::str::from_utf8(&slice[..size]).unwrap(),
            "hello"
        );
    }
}
