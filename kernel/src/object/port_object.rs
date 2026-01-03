use crate::mm::user_buffer::UserPtr;
use crate::object::capabilities::{Capability, CapabilityMask};
use crate::object::handle::Handle;
use crate::object::handle_table::HandleTable;
use crate::object::KernelObjectBase;
use crate::sched::current_task;
use crate::sync::WaitQueue;
use crate::tasks::task::Task;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use rtl::error::ErrorType;
use rtl::handle::*;
use rtl::ipc::*;
use rtl::signal::Signal;

/// Port holds weak reference to owner task, since thread may die while
/// other task has handle to it
pub struct Port {
    base: KernelObjectBase,
    task: Weak<Task>,
    queue: WaitQueue<IpcMessage<'static>>, // Kernel holds a copy, so just lie about lifetime for now
}

crate::kernel_object!(Port, Signal::MessageReady.into());

fn copy_ipc_message_from_user(
    user_msg: UserPtr<IpcMessage<'static>>,
) -> Result<IpcMessage<'static>, ErrorType> {
    let mut user_msg = user_msg.read().ok_or(ErrorType::Fault)?;

    let data = user_msg.out_arena();

    if let Some(d) = data {
        let user_buffer = UserPtr::new_array(d.as_ptr(), d.len());
        let user_buffer = user_buffer.read_on_heap()?;

        user_msg.set_out_arena(Box::leak(user_buffer));
    }

    Ok(user_msg)
}

impl Port {
    pub fn new(thread: Arc<Task>) -> Option<Arc<Self>> {
        Arc::try_new(Self {
            task: Arc::downgrade(&thread),
            queue: WaitQueue::new(),
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn full_caps() -> CapabilityMask {
        CapabilityMask::from(
            Capability::Call | Capability::Send | Capability::Receive | Capability::Wait,
        )
    }

    async fn transfer_handles_from_current(
        from: &HandleTable,
        to: &Arc<Task>,
        msg: &mut IpcMessage<'static>,
    ) -> Result<(), ErrorType> {
        let mut to_table = to.handle_table().await?;

        // TODO remove handles in case of an error
        for i in msg.handles_mut() {
            *i = to_table.add(from.find_raw_handle(*i).ok_or(ErrorType::InvalidHandle)?);
        }

        Ok(())
    }

    async fn send_impl(
        &self,
        client_msg_uptr: UserPtr<IpcMessage<'static>>,
    ) -> Result<Handle, ErrorType> {
        let mut client_msg = copy_ipc_message_from_user(client_msg_uptr)?;
        let task = self.task.upgrade().ok_or(ErrorType::TaskDead)?;
        let self_task = current_task();
        let self_table = self_task.handle_table().await?;
        let reply_port = self_table
            .find_handle::<Self>(client_msg.reply_port(), CapabilityMask::any())
            .ok_or(ErrorType::InvalidHandle)?;

        Self::transfer_handles_from_current(&self_table, &task, &mut client_msg).await?;

        // Drop self lock before waiting for the message
        drop(self_table);
        let my_port = task.handle_table().await?.add(reply_port.clone());
        client_msg.set_reply_port(my_port);
        self.produce(client_msg);

        Ok(reply_port)
    }

    pub async fn send(
        &self,
        client_msg_uptr: UserPtr<IpcMessage<'static>>,
    ) -> Result<(), ErrorType> {
        self.send_impl(client_msg_uptr).await.map(|_| ())
    }

    pub async fn call(
        &self,
        client_msg_uptr: UserPtr<IpcMessage<'static>>,
    ) -> Result<usize, ErrorType> {
        let reply_port = self.send_impl(client_msg_uptr).await?;

        reply_port
            .obj::<Self>()
            .unwrap()
            .receive(client_msg_uptr)
            .await
    }

    pub async fn reply(
        &self,
        reply_port_handle: HandleBase,
        msg: UserPtr<IpcMessage<'static>>,
    ) -> Result<(), ErrorType> {
        let self_task = current_task();
        let mut self_table = self_task.handle_table().await?;
        let reply_port = self_table
            .find::<Self>(reply_port_handle, CapabilityMask::any())
            .ok_or(ErrorType::InvalidHandle)?;

        let task = reply_port.task.upgrade().ok_or(ErrorType::TaskDead)?;

        self_table.remove(reply_port_handle);

        let mut user_msg = copy_ipc_message_from_user(msg)?;
        Self::transfer_handles_from_current(&self_table, &task, &mut user_msg).await?;
        reply_port.produce(user_msg);
        Ok(())
    }

    pub async fn reply_wait(
        &self,
        reply_port_handle: HandleBase,
        msg: UserPtr<IpcMessage<'static>>,
    ) -> Result<usize, ErrorType> {
        self.reply(reply_port_handle, msg).await?;
        self.receive(msg).await
    }

    pub async fn receive(
        &self,
        mut server_msg_uptr: UserPtr<IpcMessage<'static>>,
    ) -> Result<usize, ErrorType> {
        let mut server_msg = copy_ipc_message_from_user(server_msg_uptr)?;
        let mut arena_len = 0;

        let mut client_msg = self.queue.try_consume().ok_or(ErrorType::WouldBlock)?;

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

    fn produce(&self, message: IpcMessage<'static>) {
        self.queue.produce(message);
        self.signal(Signal::MessageReady.into());
    }
}
