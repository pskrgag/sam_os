use crate::arena::MessageArena;
use rtl::handle::*;
use libc::port::Port;
use rtl::ipc::IpcMessage;

pub struct ServerInfo<T> {
    h: Handle,
    dispatch: T,
}

pub fn server_dispatch<T>(info: &ServerInfo<T>) {
    let mut req_stack = [0u8; 1000];
    let mut res_stack = [0u8; 1000];

    let p = Port::new(info.h);

    loop {
        let req_arena = MessageArena::new_backed(req_stack.as_mut_slice());
        let res_arena = MessageArena::new_backed(res_stack.as_mut_slice());
        let mut receive_message = IpcMessage::new();

        receive_message.set_in_arena(req_arena.as_slice());
        receive_message.set_out_arena(res_arena.as_slice());

        p.receive_data(&mut receive_message);
    }
}
