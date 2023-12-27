use crate::kernel::object::handle::Handle;
use crate::make_array;
use rtl::handle::HandleBase;
use crate::kernel::object::KernelObject;
use alloc::sync::Arc;

const MAX_HANDLES: usize = 100;

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

        let h = &mut self.table[key];

        if !h.is_valid() {
            *h = obj;
            return;
        } else {
            panic!("Please refactor me....");
        }
    }

    pub fn find<T: KernelObject + Sized + 'static>(&self, hdl: HandleBase) -> Option<Arc<T>> {
        let key = hdl % MAX_HANDLES;
        let h = &self.table[key as usize];

        if !h.is_valid() {
            panic!("You are dumb");
            None
        } else {
            h.obj::<T>()
        }
    }
}
