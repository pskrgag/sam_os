use crate::{
    lib::ida::Ida,
    kernel::locking::spinlock::Spinlock,
    mm::{
        vms::Vms,
        types::*,
        paging::{
            kernel_page_table::kernel_page_table,
            page_table::PageTable,
        },
    },
    lib::collections::list::List,
    arch::{kernel_as_start, kernel_as_size},
};

pub enum TaskType {
    KernelTask,
    UserTask,
}

pub struct Task<'a> {
    name: &'a str,
    pid: usize,
    vms: Vms,
    tp: TaskType,
}


static TASK_LIST: Spinlock<List<&Task>> = Spinlock::new(List::new());

impl<'a> Task<'a> {
    pub fn new(name: &'a str, tp: TaskType) -> Self {
        let mut new = Self {
            name: name,
            pid: PID_ALLOC.lock().alloc().unwrap(),
            vms: Vms::default(),
            tp: tp,
        };

        new.init_vms();
        new
    }

    fn init_vms(&mut self) -> Option<()> {
        match &self.tp {
            TaskType::KernelTask => { self.vms = Vms::new(VirtAddr::from(kernel_as_start()), kernel_as_size(), Some(kernel_page_table().base()))?; }
            TaskType::UserTask => todo!(),
        }

        Some(())
    }
}
