use libc::port::Port;
use rtl::handle::*;
use bytemuck::*;
use rtl::error::*;
use rtl::ipc::message::*;
use ridlrt::arena::*;
use libc::port::*;

static mut SERVER_HANDLE: Option<Port> = None;

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
struct RequestHeader {
    pub num: u64,
}

pub fn sam_transport_init(h: Handle) {
    if h != HANDLE_INVALID {
        unsafe { SERVER_HANDLE = Some(Port::new(h)); }
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_FindService_in {
    pub name: ArenaPtr,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_FindService_out {
    pub h: Handle,}
pub fn sam_FindService(req: &sam_request_FindService_in, req_arena: &MessageArena, reps: &mut sam_request_FindService_out, resp_arena: Option<&mut MessageArena>) -> Result<usize, usize> {

    let mut ipc = IpcMessage::new();

    ipc.set_out_arena(req_arena.as_slice_allocated());

    if let Some(arena) = resp_arena {
        ipc.set_in_arena(arena.as_slice());
    }

    ipc.set_mid(6790964161597629750);

    unsafe { SERVER_HANDLE.as_ref().unwrap().call(&mut ipc).unwrap() };

    Ok(0)
}