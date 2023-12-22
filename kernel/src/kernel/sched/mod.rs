pub mod run_queue;

use crate::{
    arch::{self, irq, regs::Context, PAGE_SHIFT, PAGE_SIZE},
    kernel::elf::parse_elf,
    kernel::tasks::task::{init_task, kernel_task},
    kernel::tasks::thread_ep::idle_thread,
    kernel::tasks::{
        thread::{Thread, ThreadState},
    },
    mm::{types::*, vma_list::Vma},
    percpu_global,
};

use alloc::vec::Vec;
use alloc::sync::Arc;

use run_queue::RUN_QUEUE;

percpu_global! {
    static SWITCH_FIXUP: (Option<usize>, Option<usize>) = (None, None);
}

extern "C" {
    fn switch_to(from: *mut Context, to: *const Context);
}

#[repr(C)]
pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
}

macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        // const block expression to encapsulate the static

        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}

#[repr(align(4096))]
struct Aligned;

static INIT: &[u8] = include_bytes_align_as!(
    Aligned,
    "../../../../target/aarch64-unknown-none-softfloat/debug/init"
);

// Simple, Simple, Simple
//
// On cpu reset on non-boot cpus we need any sp, so we
// steal sp from per-cpu idle thread
#[no_mangle]
pub static mut IDLE_THREAD_STACK: [usize; 2] = [0, 0];

#[inline]
pub fn current() -> Option<Arc<Thread>> {
    RUN_QUEUE.per_cpu_var_get().get().current()
}

pub unsafe fn run() {
    let rq = RUN_QUEUE.per_cpu_var_get().get();

    if rq.empty() {
        return;
    }

    let cur = current();

    if let Some(c) = cur {
        if c.state() != ThreadState::NeedResched {
            return;
        }

        let next = rq.pop();
        next.set_state(ThreadState::Running);

        println!("Switching to {} --> {}", c.id(), next.id());

        let ctx = c.ctx_mut();
        let ctx_next = next.ctx_mut();

        rq.add(c.clone());

        irq::enable_all();
        switch_to(ctx as _, ctx_next as _);
        irq::disable_all();
    } else {
        let mut ctx = Context::default(); // tmp storage
        let next = rq.pop();
        let next_ctx = next.ctx_mut();

        next.set_state(ThreadState::Running);

        irq::enable_all();
        switch_to(&mut ctx as *mut _, next_ctx as *const _);
        panic!("Should not reach here");
    }
}

// ToDo: any idea fow to fix it?
pub fn init_userspace() {
    let data = parse_elf(INIT).expect("Failed to parse elf");
    let init_task = init_task();

    let init_thread = Thread::new(init_task.clone(), 0);

    let mut init_vms = init_task.vms();

    for i in data.regions {
        init_vms.vm_map(i.0, i.1, i.2).expect("Failed to map");
    }

    // Drop vms and task lock, since init_user and
    // Thread::start need them...
    drop(init_task);
    drop(init_vms);

    init_thread.init_user(data.ep);
    init_thread.start();

    // thread_table_mut()
    //     .new_user_thread("init", data)
    //     .expect("Failed to run user thread");
}

pub fn init_idle() {
    for i in 0..arch::NUM_CPUS {
        let parent = kernel_task();
        let idle = Thread::new(parent, i as u16);

        idle.init_kernel(idle_thread, ());

        unsafe {
            IDLE_THREAD_STACK[i] = 0; //PhysAddr::from(idle.stack_head().unwrap()).get();
        }

        idle.start();
    }
}
