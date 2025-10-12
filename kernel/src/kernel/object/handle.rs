use super::capabilities::CapabilityMask;
use crate::kernel::object::KernelObject;
use alloc::sync::Arc;
use core::any::TypeId;
use rtl::handle::HandleBase;

pub struct Handle {
    obj: Option<Arc<dyn KernelObject>>,
    base: HandleBase,
    rights: CapabilityMask,
}

static_assertions::const_assert!(core::mem::size_of::<Arc<i32>>() == 8);

impl Handle {
    pub const fn invalid() -> Self {
        Self {
            obj: None,
            base: 0,
            rights: CapabilityMask::invalid(),
        }
    }

    pub fn new(o: Arc<dyn KernelObject>, base: HandleBase, rights: CapabilityMask) -> Self {
        Self {
            obj: Some(o),
            base,
            rights,
        }
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

    pub fn obj_poly(&self) -> Option<Arc<dyn KernelObject>> {
        if let Some(o) = &self.obj {
            Some(o.clone())
        } else {
            None
        }
    }
}
