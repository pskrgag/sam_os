use super::handle::Handle;
use super::task_object::Task;
use super::thread_object::Thread;
use crate::kernel::locking::spinlock::Spinlock;
use crate::mm::user_buffer::UserPtr;
use crate::sched::current;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::sync::Weak;
use object_lib::object;
use rtl::error::ErrorType;
use rtl::handle::*;
use rtl::ipc::*;

/// Port holds weak reference to thread, since thread may die while
/// other task has cap to it
#[derive(object)]
pub struct Port {
    task: Weak<Task>,
    queue: Spinlock<VecDeque<IpcMessage<'static>>>, // Kernel holds a copy, so just lie about lifetime for now
    sleepers: Spinlock<LinkedList<Arc<Thread>>>,
}

fn copy_ipc_message_from_user(
    user_msg: UserPtr<IpcMessage<'static>>,
) -> Option<IpcMessage<'static>> {
    let user_msg = user_msg.read()?;

    let data = user_msg.out_arena();

    // Simply move, by bit-copy all fields, which will preserve user-addresses inside slices
    let mut msg = user_msg;

    if let Some(d) = data {
        let user_buffer = UserPtr::new_array(d.as_ptr(), d.len());
        let user_buffer = user_buffer.read_on_heap()?;

        msg.set_out_arena(Box::leak(user_buffer));
    }

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

    fn transfer_handles_from_current(to: &Task, h: &[HandleBase]) -> Option<()> {
        let cur_task = current().unwrap().task();
        let cur_table = cur_task.handle_table();

        for i in h {
            // TODO remove handles in case of an error
            let h = cur_table.find_poly(*i)?;
            let new_h = Handle::new(h);

            to.handle_table().add(new_h);
        }

        Some(())
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        use rtl::objects::port::PortInvoke;

        match PortInvoke::from_bits(args[0]).ok_or(ErrorType::NO_OPERATION)? {
            PortInvoke::CALL => {
                let mut client_msg_uptr = UserPtr::new(args[1] as *mut IpcMessage);
                let mut client_msg =
                    copy_ipc_message_from_user(client_msg_uptr).ok_or(ErrorType::FAULT)?;
                let cur = current().unwrap();

                // NOTE: Do not place it info if let Some() block, since rust does not drop the lock
                // for some weird reason
                let t = self.sleepers.lock().pop_front();
                if let Some(t) = t {
                    let task = self.task.upgrade().ok_or(ErrorType::TASK_DEAD)?;
                    let reply_port = current()
                        .unwrap()
                        .task()
                        .handle_table()
                        .find::<Self>(client_msg.reply_port())
                        .ok_or(ErrorType::INVALID_HANDLE)?;

                    reply_port.sleepers.lock().push_back(current().unwrap());

                    Self::transfer_handles_from_current(&task, client_msg.handles())
                        .ok_or(ErrorType::INVALID_HANDLE)?;

                    let h = Handle::new(reply_port.clone());

                    client_msg.set_reply_port(h.as_raw());
                    task.handle_table().add(h);

                    self.queue.lock().push_back(client_msg.clone());
                    t.wake();

                    cur.wait_send();

                    let server_msg = reply_port.queue.lock().pop_front().unwrap();

                    if let Some(d) = client_msg.in_arena() {
                        let mut ud = UserPtr::new_array(d.as_ptr(), d.len());

                        if let Some(d1) = server_msg.out_arena() {
                            ud.write_array(d1)?;
                            unsafe { drop(Box::from_raw(d1)) };
                        }
                    }

                    client_msg.add_handles(server_msg.handles());
                    client_msg_uptr.write(&client_msg)?;

                    return Ok(0);
                } else {
                    self.sleepers.lock().push_back(cur.clone());
                    cur.wait_send();
                    return self.do_invoke(args);
                };
            }
            PortInvoke::SEND_AND_WAIT => {
                let reply_port = args[1] as HandleBase;
                let cur = current().unwrap();
                let self_task = cur.task();
                let mut self_table = self_task.handle_table();

                let reply_port = self_table
                    .find::<Self>(reply_port)
                    .ok_or(ErrorType::INVALID_HANDLE)?;

                let task = reply_port.task.upgrade().ok_or(ErrorType::TASK_DEAD)?;

                self_table.remove(args[1]);
                drop(self_table);

                let user_msg = UserPtr::new(args[2] as *mut _);
                let user_msg = copy_ipc_message_from_user(user_msg).ok_or(ErrorType::FAULT)?;

                Self::transfer_handles_from_current(&task, user_msg.handles())
                    .ok_or(ErrorType::INVALID_HANDLE)?;

                reply_port.queue.lock().push_back(user_msg);
                let sleep = reply_port.sleepers.lock().pop_front().unwrap();

                sleep.wake();

                self.do_invoke(&[PortInvoke::RECEIVE.bits(), args[3]])
            }
            PortInvoke::RECEIVE => {
                let mut server_msg_uptr = UserPtr::new(args[1] as *mut _);
                let mut server_msg =
                    copy_ipc_message_from_user(server_msg_uptr).ok_or(ErrorType::FAULT)?;

                let c = current().unwrap();

                let client_msg;

                if let Some(sender) = self.sleepers.lock().pop_front() {
                    sender.wake();
                }

                loop {
                    if let Some(m) = self.queue.lock().pop_front() {
                        client_msg = m;
                        break;
                    }

                    self.sleepers.lock().push_back(c.clone());
                    c.wait_send();
                }

                // Copy arena data
                if let Some(d) = server_msg.in_arena() {
                    let mut ud = UserPtr::new_array(d.as_ptr(), d.len());

                    if let Some(d1) = client_msg.out_arena() {
                        ud.write_array(d1)?;
                        unsafe { drop(Box::from_raw(d1)) };
                    }
                }

                // Prepare message
                server_msg.set_mid(client_msg.mid());
                server_msg.set_reply_port(client_msg.reply_port());
                server_msg.add_handles(client_msg.handles());

                // Commit it to userspace
                server_msg_uptr.write(&server_msg)?;

                Ok(0)
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
