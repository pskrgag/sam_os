use super::thread_object::Thread;
use alloc::sync::Arc;
use alloc::sync::Weak;
use object_lib::object;
use rtl::error::ErrorType;
use crate::sched::current;
use crate::kernel::locking::spinlock::Spinlock;
use alloc::collections::VecDeque;
use crate::kernel::tasks::thread::ThreadState;

#[derive(Debug)]
struct IPCMessage {
    pub i: i32,
}

/// Port holds weak reference to thread, since thread may die while
/// other task has cap to it
#[derive(object)]
pub struct Port {
    thread: Weak<Thread>,
    queue: Spinlock<VecDeque<IPCMessage>>
}

impl Port {
    pub fn new(thread: Arc<Thread>) -> Arc<Self> {
        Arc::new(Self {
            thread: Arc::downgrade(&thread),
            queue: Spinlock::new(VecDeque::new()),
        })
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        use rtl::objects::port::PortInvoke;

        match PortInvoke::from_bits(args[0]).ok_or(ErrorType::NO_OPERATION)? {
            PortInvoke::CALL => {
                self.queue.lock().push_back(IPCMessage { i: 10 });
                self.thread.upgrade().ok_or(ErrorType::INVALID_ARGUMENT)?.set_state(ThreadState::Running);

                Ok(0)
            },
            PortInvoke::RECEIVE => {
                // ToDo: this actually should be resolved on level of capability
                // right which are not implemenented
                let t = self
                    .thread
                    .upgrade()
                    .expect("Receive can be called only from owning thread");

                let c = current().unwrap();

                if !Arc::ptr_eq(&t, &c) {
                    panic!();
                    return Err(ErrorType::INVALID_ARGUMENT);
                }

                let msg;

                loop {
                    if let Some(m) = self.queue.lock().pop_front() {
                        msg = m;
                        break;
                    }

                    t.wait_send();
                }

                println!("Message {:?}", msg);

                Ok(0)
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
