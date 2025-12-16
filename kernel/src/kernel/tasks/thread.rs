use crate::arch::regs::Context;
use hal::address::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadType {
    Undef,
    Kernel,
    User,
}

pub struct ThreadInner {
    arch_ctx: Option<Context>,
    stack: VirtAddr,
}

impl ThreadInner {
    pub fn new(stack: VirtAddr) -> Self {
        Self {
            arch_ctx: None,
            stack,
        }
    }

    pub fn init_context(&mut self, ep: VirtAddr, user_stack: VirtAddr, args: [usize; 3]) {
        self.arch_ctx = Some(Context::new(ep, user_stack, args));
    }

    pub fn take_context(&mut self) -> Option<Context> {
        self.arch_ctx.take()
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.arch_ctx = Some(ctx)
    }
}
