use crate::kernel::object::KernelObject;
use alloc::sync::Arc;

pub type HandleBase = u32;

const HANDLE_INVALID: HandleBase = HandleBase::MAX;

// ToDo: rigths
pub struct Handle {
    obj: Option<Arc<dyn KernelObject>>,
}

impl Handle {
    pub const fn invalid() -> Self {
        Self {
            obj: None,
        }
    }

    pub fn new<T: KernelObject>(o: Arc<dyn KernelObject>) -> Self {
        Self { obj: Some(o) }
    }

    pub const fn is_valid(&self) -> bool {
        self.obj.is_some()
    }

    pub fn obj_ptr(&self) -> usize {
        assert!(self.is_valid());

        &*self.obj.as_ref().unwrap() as *const _ as usize
    }

    pub fn obj<T: KernelObject + Sized + 'static>(&self) -> Option<&T> {
        if let Some(o) = &self.obj {
            o.as_ref().as_any().downcast_ref::<T>()
        } else {
            None
        }
    }
}
