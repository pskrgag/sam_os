use crate::{
    mm::vms::Vms,
    kernel::object::handle_table::HandleTable,
};
use alloc::sync::Arc;

const MAX_TASK_NAME: usize = 256;

struct Task {
    vms: Arc<Vms>,
    table: HandleTable,
    name: [u8; MAX_TASK_NAME],
}

impl Task {
    fn new(name: &[u8]) -> Option<Self> {
        Some(Self {
            vms: Arc::new(Vms::empty()?),
            table: HandleTable::new(),
            name: name.try_into().ok()?,
        })
    }
}
