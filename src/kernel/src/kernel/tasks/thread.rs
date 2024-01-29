use crate::{arch::regs::Context, mm::allocators::stack_alloc::StackLayout};
use rtl::vmm::types::*;

extern "C" {
    fn kernel_thread_entry_point();
    fn user_thread_entry_point();
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadState {
    Initialized,
    Running,
    WaitingMessage,
    NeedResched,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadType {
    Undef,
    Kernel,
    User,
}

pub struct ThreadInner {
    pub(crate) arch_ctx: Context,
    pub(crate) state: ThreadState,
    stack: Option<StackLayout>,
}

impl ThreadInner {
    pub fn default() -> Self {
        Self {
            state: ThreadState::Initialized,
            arch_ctx: Context::default(),
            stack: None,
        }
    }

    pub fn setup_args(&mut self, args: &[usize]) {
        if args.len() > 0 {
            self.arch_ctx.x23 = args[0];
        }

        if args.len() > 1 {
            self.arch_ctx.x24 = args[1];
        }

        if args.len() > 2 {
            self.arch_ctx.x25 = args[2];
        }

        if args.len() > 3 {
            self.arch_ctx.x26 = args[3];
        }

        if args.len() > 4 {
            self.arch_ctx.x27 = args[4];
        }
    }

    pub fn init_user(
        &mut self,
        stack: StackLayout,
        func: VirtAddr,
        user_stack: VirtAddr,
        ttbr0: usize,
    ) {
        self.arch_ctx.x21 = user_stack.bits();
        self.arch_ctx.lr = (user_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x20 = func.bits();
        self.arch_ctx.x19 = stack.stack_head().into();
        self.arch_ctx.x22 = stack.stack_head().into();
        self.arch_ctx.ttbr0 = ttbr0;

        self.stack = Some(stack);

        self.state = ThreadState::Initialized;
    }
}
