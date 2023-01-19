use crate::{
    arch::{self, regs::Context},
    mm::allocators::stack_alloc::StackLayout,
    mm::{
        types::{Address, VirtAddr},
        vms::Vms,
    },
};
use alloc::boxed::Box;

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

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadType {
    Undef,
    Kernel,
    User,
}

pub struct Thread {
    id: u16,
    arch_ctx: Context,
    state: ThreadState,
    ticks: usize,
}

impl Thread {
    pub fn new(id: u16, ep: VirtAddr, stack: VirtAddr) -> Self {
        Self {
            id: id,
            state: ThreadState::Initialized,
            arch_ctx: Context::new_thread(ep.bits(), (user_thread_entry_point as *const fn()) as usize, stack.bits()),
            ticks: RR_TICKS,
        }
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
}
