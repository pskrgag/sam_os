use crate::kernel::object::KernelObject;
use alloc::sync::Arc;

pub type HandleBase = u32;

const HANDLE_INVALID: HandleBase = HandleBase::MAX;

pub struct Handle {
    raw: HandleBase,
    obj: Option<Arc<dyn KernelObject>>,
}

impl Handle {
    pub const fn invalid() -> Self {
        Self {
            raw: HANDLE_INVALID,
            obj: None,
        }
    }

    fn obj<T: KernelObject + Sized + 'static>(&self) -> Option<&T> {
        if let Some(o) = &self.obj {
            o.as_ref().as_any().downcast_ref::<T>()
        } else {
            None
        }
    }
}
