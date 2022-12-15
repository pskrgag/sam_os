use crate::{
    arch::{self, regs::Context},
    mm::{types::VirtAddr, vms::Vms},
};
use alloc::{boxed::Box, string::String, sync::Arc};

use qrwlock::qrwlock::RwLock;

const KERN_STACK_SIZE: usize = crate::arch::PAGE_SIZE;

// Should be aligned to 2 * wordsize
#[repr(align(16))]
struct KernelStack {
    stack: [u8; KERN_STACK_SIZE],
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
    vms: Arc<RwLock<Vms>>,
    stack: Option<Box<KernelStack>>,
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
        }
    }

    pub fn set_vms(&mut self, user: bool) -> Option<()> {
        match user {
            false => {
                self.vms = Arc::new(RwLock::new(Vms::new(
                    VirtAddr::from(arch::kernel_as_start()),
                    arch::kernel_as_size(),
                    false,
                )?))
            }
            true => todo!(),
        };

        Some(())
    }

    pub fn spawn<T, F: FnMut(T) -> Option<()> + 'static>(&mut self, func: F, arg: T) {
        use crate::kernel::{
            misc::{ref_mut_to_usize, ref_to_usize},
            threading::thread_ep::kernel_thread_entry_point,
        };

        let arg = Box::new(arg);

        self.stack = Some(Box::new(KernelStack::default()));

        self.arch_ctx.sp = ref_to_usize(self.stack.as_ref().unwrap().as_ref()) + KERN_STACK_SIZE;
        self.arch_ctx.lr = ref_to_usize(&kernel_thread_entry_point::<T>);
        self.arch_ctx.x19 = ref_mut_to_usize(Box::leak(arg));
        self.arch_ctx.x20 = ref_to_usize(&func);
    }
}

impl Default for KernelStack {
    fn default() -> Self {
        Self {
            stack: [0; KERN_STACK_SIZE],
        }
    }
}
