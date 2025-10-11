use crate::kernel::object::KernelObject;
use crate::kernel::object::handle::Handle;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use rtl::handle::HandleBase;

const MAX_HANDLES: usize = 25;

// This is SHIT! Must be replaced with radix trie-like stuff,
// but for my own sake, I will leave it as simple array
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

    fn allocate_id(&mut self) -> HandleBase {
        let res = self.id;

        self.id += 1;
        res
    }

    pub fn add(&mut self, obj: Arc<dyn KernelObject>) -> HandleBase {
        let res = self.allocate_id();

        self.table.insert(res, Handle::new(obj, res));
        res
    }

    pub fn remove(&mut self, hdl: HandleBase) -> bool {
        self.table.remove(&hdl).is_some()
    }

    pub fn find<T: KernelObject + Sized + 'static>(&self, hdl: HandleBase) -> Option<Arc<T>> {
        self.table.get(&hdl).and_then(|x| x.obj::<T>())
    }

    pub fn find_poly(&self, hdl: HandleBase) -> Option<Arc<dyn KernelObject>> {
        self.table.get(&hdl).and_then(|x| x.obj_poly())
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

        // this Q/A engineer is very smart!
        test_assert!(table.find_poly(12123812398).is_none());
    }

    #[kernel_test]
    fn add_handle_find_poly() {
        let mut table = HandleTable::new();

        let t = Task::new("test".into());
        let h = Handle::new(t.clone());
        let raw = h.as_raw();

        table.add(h);
        let found = table.find_poly(raw);

        test_assert!(found.is_some());

        test_assert_eq!(
            Arc::as_ptr(&found.as_ref().unwrap()) as *const u8 as usize,
            Arc::as_ptr(&t) as usize
        );
    }

    #[kernel_test]
    fn add_handle_find() {
        let mut table = HandleTable::new();

        let t = Task::new("test".into());
        let h = Handle::new(t.clone());
        let raw = h.as_raw();

        table.add(h);
        let found = table.find::<Task>(raw);

        test_assert!(found.is_some());

        test_assert_eq!(
            Arc::as_ptr(&found.as_ref().unwrap()) as *const u8 as usize,
            Arc::as_ptr(&t) as usize
        );
    }
}
