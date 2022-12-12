use crate::{
    lib::ida::Ida,
    kernel::locking::spinlock::Spinlock,
    mm::{
        vms::Vms,
        paging::{
            kernel_page_table::kernel_page_table,
            page_table::PageTable,
        },
    },
    lib::collections::list::List,
    arch::{
        kernel_as_start,
        kernel_as_size,
        regs::Context,
    },
};
use alloc::{
    boxed::Box,
    sync::Arc,
    string::String,
};

use qrwlock::qrwlock::RwLock;

lazy_static! {
    static ref PID_ALLOC: Spinlock<Ida<1000>> = Spinlock::new(Ida::new());
}

pub enum ThreadState {
    Initialized,
    Running,
    Sleeping,
}

pub struct Thread {
    id: u16,
    arch_ctx: Context,
    name: Box<str>,
    state: ThreadState,
    vms: RwLock<Vms>,
}

impl Thread {
    pub fn new(name: &str) -> Option<Self> {
        Some(Self {
            id: PID_ALLOC.lock().alloc()?.try_into().unwrap(),
            name: String::from(name).into_boxed_str(),
            state: ThreadState::Initialized,
            vms: RwLock::new(Vms::default()),
            arch_ctx: Context::default(),
        })
    }
}
