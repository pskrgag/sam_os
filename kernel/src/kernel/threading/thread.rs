use crate::{
    arch::{self, regs::Context},
    mm::allocators::stack_alloc::StackLayout,
    mm::{types::VirtAddr, vms::Vms},
};
use alloc::{boxed::Box, string::String, sync::Arc};

use qrwlock::qrwlock::RwLock;

extern "C" {
    fn kernel_thread_entry_point();
    fn user_thread_entry_point();
}

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
    name: Box<str>,
    state: ThreadState,
    vms: Arc<RwLock<Vms>>,
    stack: Option<StackLayout>,
    kind: ThreadType,
}

lazy_static! {
    static ref KERNEL_VMS: Arc<RwLock<Vms>> = Arc::new(RwLock::new(
        Vms::new(
            VirtAddr::from(arch::kernel_as_start()),
            arch::kernel_as_size(),
            false
        )
        .expect("Failed to create kernel vms")
    ));
}

impl Thread {
    pub fn new(name: &str, id: u16) -> Self {
        Self {
            id: id,
            name: String::from(name).into_boxed_str(),
            state: ThreadState::Initialized,
            vms: Arc::new(RwLock::new(Vms::default())),
            arch_ctx: Context::default(),
            stack: None,
            kind: ThreadType::Undef,
        }
    }

    pub fn set_vms(&mut self, user: bool) -> Option<()> {
        match user {
            false => {
                self.vms = KERNEL_VMS.clone();
                self.kind = ThreadType::Kernel;
            }
            true => {
                self.vms = Arc::new(RwLock::new(Vms::new(
                    arch::user_as_start().into(),
                    arch::user_as_size(),
                    true,
                )?));

                self.kind = ThreadType::User;
            }
        };

        Some(())
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

    pub(crate) fn spawn<T>(&mut self, func: fn(T) -> Option<()>, arg: T) {
        use crate::kernel::misc::ref_mut_to_usize;

        let arg = Box::new(arg);
        let stack = StackLayout::new(3).expect("Failed to allocat stack");

        self.arch_ctx.sp = stack.stack_head().into();
        self.arch_ctx.lr = (kernel_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x19 = ref_mut_to_usize(Box::leak(arg));
        self.arch_ctx.x20 = func as usize;

        self.stack = Some(stack);

        self.state = ThreadState::Running;
    }

    pub fn spawn_user(&mut self, ep: VirtAddr) {
        let stack = StackLayout::new(3).expect("Failed to allocat stack");
        let user_stack = self
            .vms
            .write()
            .alloc_user_stack()
            .expect("Failed to allocate user stack");

        self.arch_ctx.x21 = user_stack.get() + arch::PAGE_SIZE;
        self.arch_ctx.lr = (user_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x20 = ep.get();
        self.arch_ctx.x19 = stack.stack_head().into();
        self.arch_ctx.ttbr0 = self.vms.read().ttbr0().expect("TTBR0 should be set").get();

        self.stack = Some(stack);

        self.state = ThreadState::Running;
    }

    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }

    pub fn state(&self) -> ThreadState {
        self.state
    }

    pub fn vms(&mut self) -> &RwLock<Vms> {
        &self.vms
    }

    pub fn kind(&self) -> ThreadType {
        self.kind
    }
}
