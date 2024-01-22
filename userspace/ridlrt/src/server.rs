use crate::arena::*;
use libc::port::Port;
use rtl::error::ErrorType;
use rtl::handle::*;
use rtl::ipc::IpcMessage;
use bytemuck::*;

pub trait Dispatcher {
    type DispatchReq: Copy + Zeroable;
    type DispatchResp: Copy + Zeroable;

    fn dispatch(
        &self,
        mid: usize,
        request: &Self::DispatchReq,
        req_arena: &MessageArena,
        response: &mut Self::DispatchResp,
        resp_arena: &mut MessageArena,
    );
}

pub struct ServerInfo<T: Dispatcher> {
    pub h: Handle,
    pub dispatch: T,
}

pub fn server_dispatch<T: Dispatcher>(info: &ServerInfo<T>) -> Result<(), ErrorType> {
    let mut req_stack = [0u8; 1000];
    let mut res_stack = [0u8; 1000];

    let p = Port::new(info.h);

    loop {
        let mut req_arena = MessageArena::new_backed(req_stack.as_mut_slice());
        let mut res_arena = MessageArena::new_backed(res_stack.as_mut_slice());
        let mut receive_message = IpcMessage::new();

        receive_message.set_in_arena(req_arena.as_slice());

        p.receive_data(&mut receive_message)?;

        let resp = res_arena.allocate(&T::DispatchResp::zeroed()).unwrap();
        let req = req_arena.read::<T::DispatchReq>(ArenaPtr::request_ptr::<T::DispatchReq>()).unwrap();

        let resp = resp.ptr_to_native_in_arena(&res_arena).unwrap();

        info.dispatch.dispatch(
            receive_message.mid(),
            &req,
            &req_arena,
            resp,
            &mut res_arena,
        );

        let mut reply_message = IpcMessage::new();
        reply_message.set_out_arena(res_arena.as_slice());
        reply_message.set_mid(receive_message.mid());

        p.send_data(Port::new(receive_message.reply_port()), &mut reply_message)?;
    }
}
