use super::task_object::Task;
use crate::kernel::locking::wait_queue::WaitQueue;
use crate::kernel::object::capabilities::{Capability, CapabilityMask};
use crate::mm::user_buffer::UserPtr;
use crate::sched::current;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use object_lib::object;
use rtl::error::ErrorType;
use rtl::handle::*;
use rtl::ipc::*;

/// Port holds weak reference to owner task, since thread may die while
/// other task has cap to it
#[derive(object)]
pub struct Port {
    task: Weak<Task>,
    queue: WaitQueue<IpcMessage<'static>>, // Kernel holds a copy, so just lie about lifetime for now
}

fn copy_ipc_message_from_user(
    user_msg: UserPtr<IpcMessage<'static>>,
) -> Option<IpcMessage<'static>> {
    let mut user_msg = user_msg.read()?;

    let data = user_msg.out_arena();

    if let Some(d) = data {
        let user_buffer = UserPtr::new_array(d.as_ptr(), d.len());
        let user_buffer = user_buffer.read_on_heap()?;

        user_msg.set_out_arena(Box::leak(user_buffer));
    }

    Some(user_msg)
}

impl Port {
    pub fn new(thread: Arc<Task>) -> Option<Arc<Self>> {
        Some(
            Arc::try_new(Self {
                task: Arc::downgrade(&thread),
                queue: WaitQueue::new(),
            })
            .ok()?,
        )
    }

    pub fn full_caps() -> CapabilityMask {
        CapabilityMask::from(Capability::Call | Capability::Send | Capability::Receive)
    }

    fn transfer_handles_from_current(
        to: &Task,
        msg: &mut IpcMessage<'static>,
    ) -> Result<(), ErrorType> {
        let cur_task = current().unwrap().task();
        let cur_table = cur_task.handle_table();
        let mut to_table = to.handle_table();

        // TODO remove handles in case of an error
        for i in msg.handles_mut() {
            let h = cur_table
                .find_raw_handle(*i)
                .ok_or(ErrorType::InvalidHandle)?;

            *i = to_table.add(h);
        }

        Ok(())
    }

    pub fn call(&self, mut client_msg_uptr: UserPtr<IpcMessage<'static>>) -> Result<(), ErrorType> {
        let mut client_msg = copy_ipc_message_from_user(client_msg_uptr).ok_or(ErrorType::Fault)?;
        let task = self.task.upgrade().ok_or(ErrorType::TaskDead)?;
        let reply_port = current()
            .unwrap()
            .task()
            .handle_table()
            .find_handle::<Self>(client_msg.reply_port(), CapabilityMask::any())
            .ok_or(ErrorType::InvalidHandle)?;

        Self::transfer_handles_from_current(&task, &mut client_msg)?;

        let my_port = task.handle_table().add(reply_port.clone());
        client_msg.set_reply_port(my_port);
        self.queue.produce(client_msg);

        let mut server_msg = reply_port.obj::<Self>().unwrap().queue.consume();

        if let Some(d) = client_msg.in_arena() {
            let mut ud = UserPtr::new_array(d.as_ptr(), d.len());

            if let Some(d1) = server_msg.out_arena() {
                ud.write_array(d1)?;
                unsafe { drop(Box::from_raw(d1)) };
            }
        }

        client_msg.add_handles(server_msg.handles());
        client_msg_uptr.write(&client_msg)?;
        Ok(())
    }

    pub fn send_wait(
        &self,
        reply_port_handle: HandleBase,
        msg: UserPtr<IpcMessage<'static>>,
    ) -> Result<usize, ErrorType> {
        let cur = current().unwrap();
        let self_task = cur.task();
        let mut self_table = self_task.handle_table();

        let reply_port = self_table
            .find::<Self>(reply_port_handle, CapabilityMask::any())
            .ok_or(ErrorType::InvalidHandle)?;

        let task = reply_port.task.upgrade().ok_or(ErrorType::TaskDead)?;

        self_table.remove(reply_port_handle);
        drop(self_table);

        let mut user_msg = copy_ipc_message_from_user(msg).ok_or(ErrorType::Fault)?;

        Self::transfer_handles_from_current(&task, &mut user_msg)?;

        reply_port.queue.produce(user_msg);
        self.receive(msg)
    }

    pub fn receive(
        &self,
        mut server_msg_uptr: UserPtr<IpcMessage<'static>>,
    ) -> Result<usize, ErrorType> {
        let mut server_msg = copy_ipc_message_from_user(server_msg_uptr).ok_or(ErrorType::Fault)?;
        let mut arena_len = 0;

        let mut client_msg = self.queue.consume();

        // Copy arena data
        if let Some(d) = server_msg.in_arena() {
            let mut ud = UserPtr::new_array(d.as_ptr(), d.len());

            if let Some(d1) = client_msg.out_arena() {
                arena_len = d1.len();
                ud.write_array(d1)?;
                unsafe { drop(Box::from_raw(d1)) };
            }
        }

        // Prepare message
        server_msg.set_reply_port(client_msg.reply_port());
        server_msg.add_handles(client_msg.handles());

        // Commit it to userspace
        server_msg_uptr.write(&server_msg)?;
        Ok(arena_len)
    }
}
