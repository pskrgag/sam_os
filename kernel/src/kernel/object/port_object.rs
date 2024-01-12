use super::thread_object::Thread;
use crate::kernel::locking::spinlock::Spinlock;
use crate::kernel::tasks::thread::ThreadState;
use crate::mm::user_buffer::UserBuffer;
use crate::sched::current;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::mem::size_of;
use object_lib::object;
use rtl::error::ErrorType;
use rtl::ipc::*;

/// Port holds weak reference to thread, since thread may die while
/// other task has cap to it
#[derive(object)]
pub struct Port {
    thread: Weak<Thread>,
    queue: Spinlock<VecDeque<IpcMessage<'static>>>, // Kernel holds a copy, so just lie about
                                                    // lifetime for now
}

fn copy_ipc_message_from_user(user_msg: &IpcMessage<'_>) -> Option<IpcMessage<'static>> {
    let user_buffer = UserBuffer::new(
        rtl::misc::ref_to_usize(user_msg).into(),
        size_of::<IpcMessage>(),
    );

    // TODO: Smth should be reworked.....
    let user_msg = unsafe {
        &*(&user_buffer.read_on_stack::<{ size_of::<IpcMessage>() }>()? as *const _
            as *const IpcMessage)
    };

    let h = user_msg.handles();
    let data = user_msg.data();
    let mut msg = IpcMessage::default();

    if let Some(d) = data {
        let user_buffer = UserBuffer::new(d.as_ptr().into(), d.len());
        let user_buffer = user_buffer.read_on_heap(d.len())?;

        msg.add_data(Box::leak(user_buffer));
    }

    msg.add_handles(h);

    Some(msg)
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
                todo!()
            }
            PortInvoke::SEND => {
                let msg = unsafe { rtl::misc::usize_to_ref::<IpcMessage>(args[1]) };
                let msg = copy_ipc_message_from_user(&msg).ok_or(ErrorType::FAULT)?;

                self.queue.lock().push_back(msg);
                self.thread
                    .upgrade()
                    .ok_or(ErrorType::INVALID_ARGUMENT)?
                    .set_state(ThreadState::Running);

                Ok(0)
            }
            PortInvoke::RECEIVE => {
                // ToDo: this actually should be resolved on level of capability
                // right which are not implemenented
                let t = self
                    .thread
                    .upgrade()
                    .expect("Receive can be called only from owning thread");
                let user_msg = unsafe { rtl::misc::usize_to_ref::<IpcMessage>(args[1]) };

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

                if let Some(d) = user_msg.data() {
                    let mut ud = UserBuffer::new(d.as_ptr().into(), d.len());

                    if let Some(d1) = msg.data() {
                        ud.write(d1).ok_or(ErrorType::FAULT)?;
                    }
                }

                Ok(0)
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
