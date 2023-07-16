use crate::kernel::sched::run_queue::RUN_QUEUE;
use crate::{
    arch::{PAGE_SHIFT, PAGE_SIZE},
    kernel::elf::ElfData,
    kernel::threading::{thread::Thread, ThreadRef},
    lib::ida::Ida,
    mm::types::*,
    mm::vma_list::Vma,
};

use alloc::vec::Vec;
use qrwlock::{ReadGuard, RwLock, WriteGuard};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};

lazy_static! {
    static ref THREAD_TABLE: RwLock<ThreadTable> = RwLock::new(ThreadTable::new());
}

pub struct ThreadTable {
    id_alloc: Ida<1000>,
    table: BTreeMap<u16, Arc<RwLock<Thread>>>,
}

impl ThreadTable {
    pub fn new() -> Self {
        Self {
            id_alloc: Ida::new(),
            table: BTreeMap::new(),
        }
    }

    pub fn new_kernel_thread<T>(
        &mut self,
        name: &str,
        func: fn(T) -> Option<()>,
        arg: T,
    ) -> Option<ThreadRef> {
        let new_id: u16 = self.id_alloc.alloc()?.try_into().unwrap();
        assert!(self
            .table
            .insert(new_id, Arc::new(RwLock::new(Thread::new(name, new_id))))
            .is_none());
        let thread = self.thread_by_id(new_id).unwrap();
        let mut new_thread = thread.write();

        new_thread.set_vms(false);
        new_thread.spawn(func, arg);

        drop(new_thread);
        RUN_QUEUE.per_cpu_var_get().get().add(thread);

        self.thread_by_id(new_id)
    }

    pub fn new_idle_thread<T>(
        &mut self,
        name: &str,
        func: fn(T) -> Option<()>,
        arg: T,
        cpu: usize,
    ) -> Option<ThreadRef> {
        let new_id: u16 = self.id_alloc.alloc()?.try_into().unwrap();
        assert!(self
            .table
            .insert(new_id, Arc::new(RwLock::new(Thread::new(name, new_id))))
            .is_none());
        let thread = self.thread_by_id(new_id).unwrap();
        let mut new_thread = thread.write();

        new_thread.set_vms(false);
        new_thread.spawn(func, arg);

        drop(new_thread);

        unsafe {
            RUN_QUEUE.cpu(cpu).get().add(thread);
        }

        self.thread_by_id(new_id)
    }

    pub fn thread_by_id(&self, id: u16) -> Option<ThreadRef> {
        self.table.get(&id).cloned()
    }

    pub fn new_user_thread(&mut self, name: &str, vma: ElfData) -> Option<ThreadRef> {
        let new_id: u16 = self.id_alloc.alloc()?.try_into().unwrap();
        assert!(self
            .table
            .insert(new_id, Arc::new(RwLock::new(Thread::new(name, new_id))))
            .is_none());
        let thread = self.thread_by_id(new_id).unwrap();
        let mut new_thread = thread.write();

        new_thread.set_vms(true);

        let mut vms = new_thread.vms().write();

        for i in vma.regions {
            let vma = Vma::new(i.0, i.2);
            let mut backing_store = Vec::new();
            let mut start_pa = i.1.start();

            for _ in 0..i.1.size() >> PAGE_SHIFT {
                backing_store.push(Pfn::from(start_pa));
                start_pa.add(PAGE_SIZE);
            }

            vms.add_vma_backed(vma, backing_store.as_slice());
        }

        drop(vms);

        new_thread.spawn_user(vma.ep);
        new_thread.setup_args(&["TMP"]);
        drop(new_thread);

        RUN_QUEUE.per_cpu_var_get().get().add(thread);

        self.thread_by_id(new_id)
    }
}

pub fn thread_table() -> ReadGuard<'static, ThreadTable> {
    THREAD_TABLE.read()
}

pub fn thread_table_mut() -> WriteGuard<'static, ThreadTable> {
    THREAD_TABLE.write()
}
