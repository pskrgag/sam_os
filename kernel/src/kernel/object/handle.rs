use crate::kernel::object::KernelObject;
use alloc::sync::Arc;
use uapi::handle::HandleType;

pub struct Handle {
    t: HandleType,      // To not call dynamic_cast for no reason (??)
                        // Should be reworked maybe
    obj: Option<Arc<dyn KernelObject>>,
}

impl Handle {
    pub const fn invalid() -> Self {
        Self {
            t: HandleType::Invalid,
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
