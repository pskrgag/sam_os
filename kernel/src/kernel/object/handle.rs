use super::capabilities::CapabilityMask;
use crate::kernel::object::KernelObject;
use alloc::sync::Arc;
use core::any::TypeId;

#[derive(Clone)]
pub struct Handle {
    obj: Option<Arc<dyn KernelObject>>,
    rights: CapabilityMask,
}

// dyn KernelObject is not Send by default, but all kernel objects are wrapped into Arc, so should
// be fine??? (TODO: recheck if I lied to the compiler here)
unsafe impl Send for Handle {}

impl Handle {
    pub fn new(o: Arc<dyn KernelObject>, rights: CapabilityMask) -> Self {
        Self {
            obj: Some(o),
            rights,
        }
    }

    pub fn has_capabitity(&self, caps: CapabilityMask) -> bool {
        self.rights.is_set(caps)
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

    #[allow(dead_code)]
    pub fn obj_poly(&self) -> Option<Arc<dyn KernelObject>> {
        if let Some(o) = &self.obj {
            Some(o.clone())
        } else {
            None
        }
    }
}
