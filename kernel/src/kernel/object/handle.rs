use crate::kernel::object::KernelObject;
use alloc::sync::Arc;
use core::any::TypeId;
use rtl::handle::HandleBase;

// ToDo: rigths
pub struct Handle {
    obj: Option<Arc<dyn KernelObject>>,
}

impl Handle {
    pub const fn invalid() -> Self {
        Self { obj: None }
    }

    pub fn new<T: KernelObject>(o: Arc<dyn KernelObject>) -> Self {
        Self { obj: Some(o) }
    }

    pub const fn is_valid(&self) -> bool {
        self.obj.is_some()
    }

    pub fn as_raw(&self) -> HandleBase {
        assert!(self.is_valid());

        ((Arc::as_ptr(self.obj.as_ref().unwrap()) as *const u8 as usize) & ((1 << 63) - 1))
            as HandleBase
    }

    pub fn obj<T: KernelObject + Sized + 'static>(&self) -> Option<Arc<T>> {
        if let Some(o) = &self.obj {
            if o.as_any().type_id() == TypeId::of::<T>() {
                Some(unsafe {
                    ((o as *const _ as *const u8 as *const Arc<T>)
                        .as_ref()
                        .unwrap())
                    .clone()
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
