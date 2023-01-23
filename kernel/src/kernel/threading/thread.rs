use crate::{
    arch::{self, regs::Context},
    kernel::locking::spinlock::Spinlock,
    mm::{
        allocators::slab::SlabAllocator,
        types::{Address, VirtAddr},
        vms::{Vms, VmsRef},
    },
    kernel::sched::run_queue::RUN_QUEUE,
};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::pin::Pin;
use object_lib::object;
use spin::Once;

use qrwlock::qrwlock::RwLock;

extern "C" {
    fn kernel_thread_entry_point();
    fn user_thread_entry_point();
}

const RR_TICKS: usize = 10;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadState {
    Initialized,
    Running,
    Sleeping,
    NeedResched,
}

#[derive(object)]
pub struct Thread {
    id: u16,
    arch_ctx: Context,
    state: ThreadState,
    ticks: usize,
    vms: VmsRef,
}

impl Thread {
    pub fn new(id: u16, ep: VirtAddr, stack: VirtAddr, vms: Option<VmsRef>) -> Option<ThreadRef> {
        Some(Self::construct(Self {
            id: id,
            state: ThreadState::Initialized,
            arch_ctx: Context::new_thread(
                ep.bits(),
                (user_thread_entry_point as *const fn()) as usize,
                stack.bits(),
            ),
            ticks: RR_TICKS,
            vms: match vms {
                Some(v) => v,
                None => Vms::empty()?,
            },
        }))
    }

    // idle thread which do nothing but waits for an interrupt
    pub fn kernel_thread(id: u16, f: fn()) -> Option<ThreadRef> {
        let ep = (kernel_thread_entry_point as *const fn()) as usize;

        Some(Self::construct(Self {
            id: id,
            state: ThreadState::Initialized,
            arch_ctx: Context::new_kernel_thread(
                ep,
                (f as *const fn()) as usize,
                crate::mm::allocators::stack_alloc::KERNEL_STACKS
                    .per_cpu_var_get()
                    .get()
                    .unwrap()
                    .bits(),
            ),
            ticks: RR_TICKS,
            vms: Vms::empty()?,
        }))
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn ctx_mut(&mut self) -> &mut Context {
        &mut self.arch_ctx
    }

    pub fn ctx(&mut self) -> &Context {
        &self.arch_ctx
    }

    pub fn tick(&mut self) {
        self.ticks -= 1;

        if self.ticks == 0 {
            self.state = ThreadState::NeedResched;
            self.ticks = RR_TICKS;
        }
    }

    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }

    pub fn state(&self) -> ThreadState {
        self.state
    }

    pub fn resume(r: ThreadRef) {
        RUN_QUEUE.per_cpu_var_get().get().add(r);
    }
}
