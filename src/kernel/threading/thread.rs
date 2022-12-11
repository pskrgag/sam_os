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
    arch::{
        kernel_as_start,
        kernel_as_size,
        regs::Context,
    },
};
use alloc::{
    boxed::Box,
    sync::Arc,
};

lazy_static! {
    static ref PID_ALLOC: Spinlock<Ida<1000>> = Spinlock::new(Ida::new());
}

pub enum ThreadState {
    Running,
    Sleeping,
}

pub struct Thread {
    id: u16,
    arch_ctx: Context,
    name: Box<str>,
    state: ThreadState, 
}
