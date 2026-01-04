use super::factory::init_self_factory;
use super::syscalls::Syscall;
use super::vmm::vms::init_self_vms;
use alloc::collections::BTreeMap;
use hal::arch::PAGE_SIZE;
use heapless::String;
use heapless::Vec;
use postcard::from_bytes;
use rtl::handle::MAX_HANDLE_NAME;
use rtl::{error::ErrorType, handle};
use spin::Once;

/// Owning RAII wrapper around handle
#[derive(Debug)]
pub struct Handle(handle::Handle);

impl Handle {
    pub fn new(h: handle::Handle) -> Self {
        Self(h)
    }

    /// # SAFETY
    /// don't use it, unless you know what you are doing
    pub unsafe fn as_raw(&self) -> handle::Handle {
        self.0
    }

    pub fn clone_handle(&self) -> Result<Self, ErrorType> {
        Syscall::clone_handle(self)
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        Syscall::close_handle(self.0).unwrap();
    }
}

pub type HandleName = String<MAX_HANDLE_NAME>;
type Pair = (HandleName, handle::Handle);

static OTHER_HANDLES: Once<BTreeMap<HandleName, Handle>> = Once::new();

pub fn parse_handle_table(ptr: *const u8) -> Result<Option<Handle>, ErrorType> {
    unsafe {
        let page_slice = core::slice::from_raw_parts(ptr, PAGE_SIZE);
        let vec: Vec<Pair, 500> = from_bytes(page_slice).unwrap();
        let mut boot_handle = None;
        let mut map = BTreeMap::new();

        for entry in vec {
            match entry.0.as_str() {
                "VMS" => init_self_vms(Handle::new(entry.1)),
                "FACTORY" => init_self_factory(Handle::new(entry.1)),
                "BOOT" => boot_handle = Some(Handle::new(entry.1)),
                _ => {
                    map.insert(entry.0, Handle::new(entry.1));
                }
            }
        }

        OTHER_HANDLES.call_once(|| map);
        Ok(boot_handle)
    }
}
