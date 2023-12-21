use crate::kernel::object::handle::Handle;
use crate::make_array;

const MAX_HANDLES: usize = 10;

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
        let key = obj.obj_ptr() % MAX_HANDLES;
        let mut key_iter = key;

        while {
            let h = &mut self.table[key];

            if !h.is_valid() {
                *h = obj;
                return;
            }

            key_iter = (key_iter + 1) % MAX_HANDLES;
            key_iter != key
        } { };

        panic!("hehe");
    }
}
