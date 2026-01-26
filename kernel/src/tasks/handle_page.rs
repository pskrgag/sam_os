use super::task::Task;
use adt::Vec;
use crate::object::handle::Handle;
use alloc::sync::Arc;
use hal::address::VirtualAddress;
use hal::arch::PAGE_SIZE;
use heapless::String;
use postcard::to_slice;
use rtl::error::ErrorType;
use rtl::handle::{HandleBase, MAX_HANDLE_NAME};
use rtl::vmm::MappingType;

pub type HandleName = String<MAX_HANDLE_NAME>;
type Pair = (HandleName, HandleBase);

/// User-space page of transferred handles
pub struct HandlePage {
    pairs: Vec<Pair>,
    task: Arc<Task>,
}

impl HandlePage {
    pub fn new(task: Arc<Task>) -> Self {
        Self {
            pairs: Vec::new(),
            task,
        }
    }

    pub async fn push(&mut self, handle: Handle, name: HandleName) -> Result<(), ErrorType> {
        let mut table = self.task.handle_table().await?;
        let raw = table.add(handle);

        // TODO: maybe add checks for collisions?
        self.pairs.try_push((name, raw))
    }

    pub async fn into_ptr(self) -> Result<*mut u8, ErrorType> {
        let full_size = self.pairs.len() * core::mem::size_of::<Pair>();
        let full_size = full_size.next_multiple_of(PAGE_SIZE);

        let mut ptr = self
            .task
            .vms()
            .vm_allocate(full_size, MappingType::Data)
            .await?;

        self.task.with_attached_task(|| unsafe {
            let new_slice = ptr.as_slice_mut::<u8>(self.pairs.len() * core::mem::size_of::<Pair>());

            // TODO: this won't work with PAN
            to_slice(self.pairs.as_slice(), new_slice).unwrap();
        });

        // TODO: protect as RODATA
        Ok(ptr.to_raw_mut())
    }
}
