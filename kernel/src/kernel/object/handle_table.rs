use crate::kernel::object::handle::Handle;
use crate::lib::collections::ertrie::RadixTrie;
use crate::mm::types::{Page, VirtAddr};
use alloc::sync::Arc;
use object_lib::object;
use uapi::handle::UHandle;


#[derive(object)]
pub struct HandleTable {
    table: RadixTrie<3, Handle>,
}

impl HandleTable {
    /// Create empty handle table
    pub fn new(p: Page) -> HandleTableRef {
        Self::construct(Self {
            table: RadixTrie::new(p),
        })
    }

    pub fn find(h: UHandle) -> Option<&Handle> {

    }
}
