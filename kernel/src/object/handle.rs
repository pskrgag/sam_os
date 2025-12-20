use super::capabilities::CapabilityMask;
use crate::object::KernelObject;
use alloc::sync::Arc;
use core::any::TypeId;

#[derive(Clone)]
pub struct Handle {
    obj: Option<Arc<dyn KernelObject + Send + Sync>>,
    rights: CapabilityMask,
}

impl Handle {
    pub fn new(o: Arc<dyn KernelObject + Send + Sync>, rights: CapabilityMask) -> Self {
        Self {
            obj: Some(o),
            rights,
        }
    }

    pub fn has_capabitity(&self, caps: CapabilityMask) -> bool {
        self.rights.is_set(caps)
    }

    pub fn obj<T: KernelObject + Sized + 'static + Send>(&self) -> Option<Arc<T>> {
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

    pub fn obj_poly(&self) -> Option<Arc<dyn KernelObject + Send + Sync>> {
        if let Some(o) = &self.obj {
            Some(o.clone())
        } else {
            None
        }
    }
}
