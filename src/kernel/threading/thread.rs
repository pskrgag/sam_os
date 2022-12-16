use crate::{
    arch::{self, regs::Context},
    mm::{types::VirtAddr, vms::Vms},
};
use alloc::{boxed::Box, string::String, sync::Arc};

use qrwlock::qrwlock::RwLock;

const KERN_STACK_SIZE: usize = crate::arch::PAGE_SIZE / 2;

extern "C" {
    fn kernel_thread_entry_point();
}

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
        use crate::kernel::misc::{ref_mut_to_usize, ref_to_usize};

        let arg = Box::new(arg);

        self.stack = Some(Box::new(KernelStack::default()));

        self.arch_ctx.sp = ref_to_usize(self.stack.as_ref().unwrap().as_ref()) + KERN_STACK_SIZE;
        self.arch_ctx.lr = (kernel_thread_entry_point as *const fn()) as usize;
        self.arch_ctx.x19 = ref_mut_to_usize(Box::leak(arg));
        self.arch_ctx.x20 = func as usize;

        println!("Thread sp 0x{:x}", self.arch_ctx.sp);
        println!("Thread entry point {:p}", self.arch_ctx.lr as *const usize);
        println!(
            "Thread {:p} {:p} {}",
            self as *const _,
            &self.arch_ctx as *const _,
            core::mem::size_of::<Self>()
        );

        self.state = ThreadState::Running;
    }
}

impl Default for KernelStack {
    fn default() -> Self {
        Self {
            stack: [0; KERN_STACK_SIZE],
        }
    }
}
