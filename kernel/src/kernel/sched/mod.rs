use crate::kernel::object::thread_object::Thread;
use crate::percpu_global;
use crate::{
    arch::irq, arch::regs::Context, kernel::elf::parse_elf, kernel::tasks::task::init_task,
    kernel::tasks::thread::ThreadState,
};
use alloc::sync::Arc;
use rtl::locking::fake_lock::FakeLock;
use run_queue::RunQueue;

pub mod run_queue;

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

#[repr(align(0x1000))]
struct Aligned;

static INIT: &[u8] = include_bytes_align_as!(
    Aligned,
    "../../../../target/aarch64-unknown-none-softfloat/debug/nameserver"
);

// Simple, Simple, Simple
//
// On cpu reset on non-boot cpus we need any sp, so we
// steal sp from per-cpu idle thread
#[no_mangle]
pub static mut IDLE_THREAD_STACK: [usize; 2] = [0, 0];

#[inline]
pub fn current() -> Option<Arc<Thread>> {
    crate::arch::current::get_current()
}

pub struct Scheduler {
    rq: RunQueue,
}

percpu_global!(
    pub static SCHEDULER: FakeLock<Scheduler> = FakeLock::new(Scheduler::new());
);

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            rq: RunQueue::new(),
        }
    }

    pub fn schedule(&mut self) {
        // Fix up queue based on recent events
        self.rq.move_by_pred(|x| x.state() == ThreadState::Running);

        // If rq is empty -- do noting
        // if self.rq.empty(){
        //     return;
        // }

        if let Some(cur) = current() {
            match cur.state() {
                ThreadState::Running => return, // Just timer tick
                ThreadState::NeedResched => self.rq.add_running(cur.clone()),
                ThreadState::WaitingMessage => self.rq.add_sleeping(cur.clone()),
                _ => panic!("Running scheduler in unexected state"),
            }

            if let Some(next) = self.rq.pop() {
                next.set_state(ThreadState::Running);

                println!("Switching to {} --> {}", cur.id(), next.id());

                unsafe {
                    let ctx = cur.ctx_mut();
                    let ctx_next = next.ctx_mut();

                    crate::arch::current::set_current(next.clone());

                    switch_to(ctx as _, ctx_next as _);
                }
            } else if cur.state() == ThreadState::WaitingMessage {
                crate::drivers::timer::disable();
                unsafe { core::arch::asm!("wfi") };
            }
        } else {
            let mut ctx = Context::default(); // tmp storage
            let next = self.rq.pop().expect("Rq must not be empty at that moment");
            let next_ctx = unsafe { next.ctx_mut() };

            next.set_state(ThreadState::Running);

            crate::arch::current::set_current(next.clone());

            // We come here only for 1st process, so we need to turn on irqs
            unsafe {
                irq::enable_all();
                switch_to(&mut ctx as *mut _, next_ctx as *const _);
            }
            panic!("Should not reach here");
        }
    }

    pub fn add_thread(&mut self, t: Arc<Thread>) {
        match t.state() {
            ThreadState::Running => self.rq.add_running(t),
            ThreadState::WaitingMessage => self.rq.add_sleeping(t),
            _ => panic!("Called on wrong thread state"),
        }
    }
}

pub fn run() {
    let scheduler = SCHEDULER.per_cpu_var_get().get();
    scheduler.schedule();
}

pub fn add_thread(t: Arc<Thread>) {
    let scheduler = SCHEDULER.per_cpu_var_get().get();
    scheduler.add_thread(t);
}

pub fn init_userspace() {

    use rtl::vmm::types::*;
    assert!((INIT.as_ptr() as usize).is_page_aligned());

    let data = parse_elf(INIT).expect("Failed to parse elf");
    let init_task = init_task();

    let init_thread = Thread::new(init_task.clone(), 0);

    let init_vms = init_task.vms();

    for mut i in data.regions {
        println!("{:?} {:?}", i.0, i.1);
        i.0.align_page();
        i.1.align_page();
        init_vms.vm_map(i.0, i.1, i.2).expect("Failed to map");
    }

    println!("Started userspace...");

    init_thread.init_user(data.ep);

    init_task.add_initial_thread(init_thread, rtl::handle::HANDLE_INVALID);
    init_task.start();
}
