use crate::kernel::object::KernelObject;
use crate::kernel::object::capabilities::CapabilityMask;
use crate::kernel::object::handle::Handle;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use rtl::handle::HandleBase;

const MAX_HANDLES: usize = 25;

pub struct HandleTable {
    table: BTreeMap<HandleBase, Handle>,
    id: usize,
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            table: BTreeMap::new(),
            id: 0,
        }
    }

    // TODO: This is something that tastes like shit, but whatever
    fn allocate_id(&mut self) -> HandleBase {
        let res = self.id;
        let (new, overflow) = self.id.overflowing_add(1);

        assert!(!overflow);
        self.id = new;
        res
    }

    #[must_use]
    pub fn add(&mut self, handle: Handle) -> HandleBase {
        let res = self.allocate_id();

        self.table.insert(res, handle);
        res
    }

    pub fn remove(&mut self, hdl: HandleBase) -> bool {
        self.table.remove(&hdl).is_some()
    }

    pub fn find<T: KernelObject + Sized + 'static>(
        &self,
        hdl: HandleBase,
        rights: CapabilityMask,
    ) -> Option<Arc<T>> {
        self.table
            .get(&hdl)
            .filter(|x| x.has_capabitity(rights))
            .and_then(|x| x.obj::<T>())
    }

    pub fn find_handle<T: KernelObject + Sized + 'static>(
        &self,
        hdl: HandleBase,
        rights: CapabilityMask,
    ) -> Option<Handle> {
        self.table
            .get(&hdl)
            .filter(|x| x.has_capabitity(rights))
            .cloned()
    }

    pub fn find_poly(
        &self,
        hdl: HandleBase,
        rights: CapabilityMask,
    ) -> Option<Arc<dyn KernelObject>> {
        self.table
            .get(&hdl)
            .filter(|x| x.has_capabitity(rights))
            .and_then(|x| x.obj_poly())
    }

    pub fn find_raw_handle(&self, hdl: HandleBase) -> Option<Handle> {
        self.table.get(&hdl).cloned()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::kernel::object::handle::Handle;
    use crate::kernel::object::task_object::Task;
    use crate::*;
    use alloc::sync::Arc;
    use test_macros::*;

    #[kernel_test]
    fn find_smth() {
        let table = HandleTable::new();

        test_assert!(
            table
                .find_poly(12123812398, CapabilityMask::any())
                .is_none()
        );
    }

    #[kernel_test]
    fn add_handle_find_poly() {
        let mut table = HandleTable::new();

        let t = Task::new("test".into()).unwrap();
        let h = Handle::new(t.clone(), CapabilityMask::any());

        let hdl = table.add(h);
        let found = table.find_poly(hdl, CapabilityMask::any());

        test_assert!(found.is_some());

        test_assert_eq!(
            Arc::as_ptr(&found.as_ref().unwrap()) as *const u8 as usize,
            Arc::as_ptr(&t) as usize
        );
    }

    #[kernel_test]
    fn add_handle_find() {
        let mut table = HandleTable::new();

        let t = Task::new("test".into()).unwrap();
        let h = Handle::new(t.clone(), CapabilityMask::any());

        let hdl = table.add(h);
        let found = table.find::<Task>(hdl, CapabilityMask::any());
        test_assert!(found.is_some());

        test_assert_eq!(
            Arc::as_ptr(&found.as_ref().unwrap()) as *const u8 as usize,
            Arc::as_ptr(&t) as usize
        );
    }
}
