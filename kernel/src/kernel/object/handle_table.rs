use crate::kernel::object::handle::Handle;

use alloc::collections::LinkedList;

// Handle Tabled owns handles
pub struct HandleTable {
    table: LinkedList<Handle>,
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            table: LinkedList::new(),
        }
    }
}
