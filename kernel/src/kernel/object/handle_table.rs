use crate::kernel::object::handle::Handle;
use crate::make_array;

use super::KernelObject;

const MAX_HANDLES: usize = 1000;

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

    pub fn add<T: KernelObject>(&mut self, obj: &mut T) -> &mut Handle {
        let h = self.table.iter().find(|&x| !x.is_valid()).unwrap();

        todo!()
    }
}
