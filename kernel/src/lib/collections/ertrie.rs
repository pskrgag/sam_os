use crate::arch::PAGE_SIZE;
use crate::mm::types::*;
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::mem::size_of;

/* Helper comment for development
 *
 * Each level is PAGE_SIZE.
 *
 * Leaf level: PAGE_SIZE / sizeof T
 * Nonleaf: PAGE_SIZE / sizeof void*
 */
pub struct RadixTrie<const depth: usize, T> {
    root: Page,
    _p: PhantomData<T>,
}

struct RadixEntrie<T: Sized> {
    el: T,
}

impl<T: Sized> RadixEntrie<T> {
    pub fn new(val: T) -> Self {
        Self { el: val }
    }

    pub unsafe fn in_place(va: VirtAddr) -> Box<Self> {
        Box::from_raw(va.to_raw_mut::<Self>())
    }

    pub fn is_null(s: &Box<Self>) -> bool
    where
        [(); size_of::<T>()]:,
    {
        let null_arr = [0_u8; size_of::<T>()];
        let sl =
            unsafe { core::slice::from_raw_parts(&**s as *const _ as *const _, size_of::<T>()) };

        null_arr == sl
    }

    pub fn val(self) -> T {
        self.el
    }
}

impl<const depth: usize, T> RadixTrie<depth, T>
where
    [(); size_of::<T>()]:,
{
    const BITS_PER_LEAF: usize = (PAGE_SIZE / core::mem::size_of::<T>()).ilog2() as usize;
    const BITS_PER_NON_LEAF: usize =
        (PAGE_SIZE / core::mem::size_of::<*const u8>()).ilog2() as usize;

    pub const fn index_mask<const IDX: usize>() -> usize {
        use crate::kernel::misc::genmask;

        if IDX == depth {
            genmask(Self::BITS_PER_LEAF, 0)
        } else {
            genmask(
                Self::BITS_PER_NON_LEAF,
                Self::BITS_PER_LEAF + Self::BITS_PER_NON_LEAF * (IDX - 1),
            )
        }
    }

    pub fn index<const IDX: usize>(v: VirtAddr) -> usize
    where
        [u8; depth - IDX - 1]: Sized,
    {
        let raw = v.get();

        raw & Self::index_mask::<IDX>()
    }

    pub const fn index_mask_array() -> [usize; depth] {
        let a = [0_usize; depth];
        let mut idx = 0;

        while idx < depth {
            a[idx] = match idx {
                1 => Self::index_mask::<1>(),
                2 => Self::index_mask::<2>(),
                3 => Self::index_mask::<3>(),
                4 => Self::index_mask::<4>(),
                _ => unreachable!(),
            };

            idx += 1;
        }

        a
    }

    pub const fn new(p: Page) -> Self {
        Self {
            root: p,
            _p: PhantomData,
        }
    }

    fn do_for_non_leaf<F: FnMut(&mut Box<RadixEntrie<*const u8>>) -> bool>(
        &self,
        mut v: usize,
        f: F,
    ) -> usize {
        let mut root = self.root.va();
        let masks = Self::index_mask_array();
        let mut count = 0;

        for i in 0..depth - 1 {
            let e = unsafe {
                RadixEntrie::<*const u8>::in_place(VirtAddr::from_raw(
                    root.to_raw::<u8>().add(v & masks[i]),
                ))
            };

            if f(&mut e) {
                count += 1;
            }

            if RadixEntrie::<*const u8>::is_null(&e) {
                return count;
            }

            root = VirtAddr::from_raw((*e).val());
        }

        count
    }

    pub fn expand<F: Fn() -> Option<Page>>(&self, id: usize, f: F) -> usize {
        self.do_for_non_leaf(id, |x| {
            if !RadixEntrie::<*const u8>::is_null(x) {
                false
            } else {
                if let Some(p) = f() {
                    **x = RadixEntrie::<*const u8>::new(p.va().to_raw_mut());
                    true
                } else {
                    false
                }
            }
        })
    }

    // It still can be null. Let caller do what it wants to do
    pub fn find_slot(&self, mut v: usize) -> Option<Box<RadixEntrie<T>>> {
        let mut root = self.root.va();
        let masks = Self::index_mask_array();

        // Handle all cases cases except for leaf
        for i in 0..depth - 1 {
            let e = unsafe {
                RadixEntrie::<*const u8>::in_place(VirtAddr::from_raw(
                    root.to_raw::<u8>().add(v & masks[i]),
                ))
            };

            if RadixEntrie::<*const u8>::is_null(&e) {
                return None;
            }

            root = VirtAddr::from_raw((*e).val());
        }

        let e = unsafe {
            RadixEntrie::<T>::in_place(VirtAddr::from_raw(
                root.to_raw::<u8>().add(v & masks[depth - 1]),
            ))
        };

        Some(e)
    }
}
