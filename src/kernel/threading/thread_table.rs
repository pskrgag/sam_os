use crate::{kernel::threading::thread::Thread, lib::ida::Ida};
use qrwlock::qrwlock::{ReadGuard, RwLock, WriteGuard};

use alloc::collections::btree_map::BTreeMap;

lazy_static! {
    static ref THREAD_TABLE: RwLock<ThreadTable> = RwLock::new(ThreadTable::new());
}

pub struct ThreadTable {
    id_alloc: Ida<1000>,
    table: BTreeMap<u16, RwLock<Thread>>,
}

impl ThreadTable {
    pub fn new() -> Self {
        Self {
            id_alloc: Ida::new(),
            table: BTreeMap::new(),
        }
    }

    pub fn new_thread(&mut self, name: &str) -> Option<&RwLock<Thread>> {
        let new_id: u16 = self.id_alloc.alloc()?.try_into().unwrap();
        let new_thread = RwLock::new(Thread::new(name, new_id));

        assert!(self.table.insert(new_id, new_thread).is_none());
        self.thread_by_id(new_id)
    }

    pub fn thread_by_id(&self, id: u16) -> Option<&RwLock<Thread>> {
        self.table.get(&id)
    }
}

pub fn thread_table() -> ReadGuard<'static, ThreadTable> {
    THREAD_TABLE.read()
}

pub fn thread_table_mut() -> WriteGuard<'static, ThreadTable> {
    THREAD_TABLE.write()
}
