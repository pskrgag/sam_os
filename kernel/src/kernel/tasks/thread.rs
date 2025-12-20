use crate::arch::regs::Context;
use core::task::Waker;
use hal::address::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThreadType {
    Undef,
    Kernel,
    User,
}

pub struct ThreadInner {
    arch_ctx: Option<Context>,
    stack: MemRange<VirtAddr>,
    waker: Option<Waker>,
}

impl ThreadInner {
    pub fn new(stack: MemRange<VirtAddr>) -> Self {
        Self {
            arch_ctx: None,
            stack,
            waker: None,
        }
    }

    pub fn set_waker(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }

    pub fn wake(&mut self) {
        self.waker.take().map(|x| x.wake());
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
