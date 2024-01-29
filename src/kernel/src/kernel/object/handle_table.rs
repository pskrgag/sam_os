use crate::kernel::object::handle::Handle;
use crate::kernel::object::KernelObject;
use crate::make_array;
use alloc::sync::Arc;
use rtl::handle::HandleBase;

const MAX_HANDLES: usize = 20;

// This is SHIT! Must be replaced with radix trie-like stuff,
// but for my own sake, I will leave it as simple array
pub struct HandleTable {
    table: [Handle; MAX_HANDLES],
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            table: unsafe { make_array!(MAX_HANDLES, |_i| Handle::invalid()) },
        }
    }

    pub fn add(&mut self, obj: Handle) {
        let key = obj.as_raw() as usize % MAX_HANDLES;
        let start = key;
        let mut idx = start;

        while {
            let h = &mut self.table[idx];

            if !h.is_valid() {
                *h = obj;
                return;
            }

            idx = (idx + 1) % MAX_HANDLES;
            idx != start
        } {}

        panic!("Please refactor me...");
    }

    pub fn remove(&mut self, hdl: HandleBase) {
        let key = hdl % MAX_HANDLES;

        let start = key;
        let mut idx = start;

        while {
            let h = &mut self.table[idx as usize];

            if h.is_valid() && h.as_raw() == hdl {
                *h = Handle::invalid();
                return;
            }

            idx = (idx + 1) % MAX_HANDLES;
            idx != start
        } {}
    }

    // ToDo factor out finding loop into own function
    pub fn find<T: KernelObject + Sized + 'static>(&self, hdl: HandleBase) -> Option<Arc<T>> {
        let key = hdl % MAX_HANDLES;

        let start = key;
        let mut idx = start;

        while {
            let h = &self.table[idx as usize];

            if h.is_valid() && h.as_raw() == hdl {
                return h.obj::<T>();
            }

            idx = (idx + 1) % MAX_HANDLES;
            idx != start
        } {}

        None
    }

    pub fn find_poly(&self, hdl: HandleBase) -> Option<Arc<dyn KernelObject>> {
        let key = hdl % MAX_HANDLES;

        let start = key;
        let mut idx = start;

        while {
            let h = &self.table[idx as usize];

            if h.is_valid() && h.as_raw() == hdl {
                return h.obj_poly();
            }

            idx = (idx + 1) % MAX_HANDLES;
            idx != start
        } {}

        None
    }
}
