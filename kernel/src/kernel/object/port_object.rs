use super::handle::Handle;
use super::task_object::Task;
use super::thread_object::Thread;
use crate::kernel::locking::spinlock::Spinlock;
use crate::mm::user_buffer::UserBuffer;
use crate::sched::current;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::mem::size_of;
use object_lib::object;
use rtl::error::ErrorType;
use rtl::handle::*;
use rtl::ipc::*;

/// Port holds weak reference to thread, since thread may die while
/// other task has cap to it
#[derive(object)]
pub struct Port {
    task: Weak<Task>,
    queue: Spinlock<VecDeque<IpcMessage<'static>>>, // Kernel holds a copy, so just lie about
    // lifetime for now
    sleepers: Spinlock<LinkedList<Arc<Thread>>>,
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
    let data = user_msg.in_data();

    let mut msg = IpcMessage::default();

    if let Some(d) = data {
        let user_buffer = UserBuffer::new(d.as_ptr().into(), d.len());
        let user_buffer = user_buffer.read_on_heap(d.len())?;

        msg.add_data_raw(Box::leak(user_buffer));
    }

    msg.add_handles(h);
    if let Some(d) = user_msg.out_data() {
        msg.set_out_data_raw(d);
    }

    msg.set_reply_port(user_msg.reply_port());

    Some(msg)
}

impl Port {
    pub fn new(thread: Arc<Task>) -> Arc<Self> {
        Arc::new(Self {
            task: Arc::downgrade(&thread),
            queue: Spinlock::new(VecDeque::new()),
            sleepers: Spinlock::new(LinkedList::new()),
        })
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        use rtl::objects::port::PortInvoke;

        match PortInvoke::from_bits(args[0]).ok_or(ErrorType::NO_OPERATION)? {
            PortInvoke::CALL => {
                let msg = unsafe { rtl::misc::usize_to_ref::<IpcMessage>(args[1]) };
                let mut msg = copy_ipc_message_from_user(&msg).ok_or(ErrorType::FAULT)?;

                if let Some(t) = self.sleepers.lock().pop_front() {
                    let task = self.task.upgrade().ok_or(ErrorType::TASK_DEAD)?;
                    let reply_port = current()
                        .unwrap()
                        .task()
                        .handle_table()
                        .find::<Self>(msg.reply_port())
                        .ok_or(ErrorType::INVALID_HANDLE)?;

                    reply_port.sleepers.lock().push_back(current().unwrap());

                    let h = Handle::new(reply_port.clone());

                    msg.set_reply_port(h.as_raw());
                    task.handle_table().add(h);

                    self.queue.lock().push_back(msg.clone());
                    t.wake();

                    current().unwrap().wait_send();

                    let reply_msg = reply_port.queue.lock().pop_front().unwrap();

                    if let Some(d) = msg.out_data() {
                        let mut ud = UserBuffer::new(d.as_ptr().into(), d.len());

                        if let Some(d1) = reply_msg.in_data() {
                            ud.write(d1).ok_or(ErrorType::FAULT)?;
                        }
                    }

                    Ok(())
                } else {
                    Err(ErrorType::TRY_AGAIN)
                }?;

                Ok(0)
            }
            PortInvoke::SEND => {
                let reply_port = args[1] as HandleBase;
                let cur = current().unwrap();
                let self_task = cur.task();
                let mut self_table = self_task.handle_table();

                let reply_port = self_table
                    .find::<Self>(reply_port)
                    .ok_or(ErrorType::INVALID_HANDLE)?;

                self_table.remove(args[1]);

                let user_msg = unsafe { rtl::misc::usize_to_ref::<IpcMessage>(args[2]) };
                let user_msg = copy_ipc_message_from_user(&user_msg).ok_or(ErrorType::FAULT)?;

                reply_port.queue.lock().push_back(user_msg);
                let sleep = reply_port.sleepers.lock().pop_front().unwrap();

                drop(self_table);

                sleep.wake();
                cur.self_yield();

                Ok(0)
            }
            PortInvoke::RECEIVE => {
                let user_msg = unsafe { rtl::misc::usize_to_ref::<IpcMessage>(args[1]) };
                let user_msg = copy_ipc_message_from_user(&user_msg).ok_or(ErrorType::FAULT)?;

                let c = current().unwrap();

                let msg;

                loop {
                    if let Some(m) = self.queue.lock().pop_front() {
                        msg = m;
                        break;
                    }

                    self.sleepers.lock().push_back(c.clone());
                    c.wait_send();
                }

                if let Some(d) = user_msg.out_data() {
                    let mut ud = UserBuffer::new(d.as_ptr().into(), d.len());

                    if let Some(d1) = msg.in_data() {
                        ud.write(d1).ok_or(ErrorType::FAULT)?;
                    }
                }

                Ok(msg.reply_port())
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
