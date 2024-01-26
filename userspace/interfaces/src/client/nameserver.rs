use libc::port::Port;
use rtl::handle::*;
use bytemuck::*;
use rtl::error::*;
use rtl::ipc::message::*;
use ridlrt::arena::*;
use libc::port::*;

static mut SERVER_HANDLE: Option<Port> = None;

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
    pub error: ErrorType,    pub h: Handle,}
pub fn sam_FindService(req: &mut sam_request_FindService_in, req_arena: &MessageArena, resp: &mut sam_request_FindService_out, resp_arena: &mut MessageArena) -> Result<usize, usize> {

    let mut ipc = IpcMessage::new();

    ipc.set_out_arena(req_arena.as_slice_allocated());

    // if let Some(arena) = resp_arena {
        ipc.set_in_arena(resp_arena.as_slice());
    // }

    ipc.set_mid(6790964161597629750);

    

    unsafe { SERVER_HANDLE.as_ref().unwrap().call(&mut ipc) }?;
    let h = ipc.handles();
let mut resp_ = resp_arena.read::<sam_request_FindService_out>(ArenaPtr::request_ptr::<sam_request_FindService_out>()).unwrap();
                     let error = resp_.error;
                     if error != 0.into() {
                        return Err(error.into());
                     }
                    resp_.h = h[0];

    *resp = resp_;

    Ok(0)
}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_RegisterService_in {
    pub name: ArenaPtr,    pub h: Handle,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_RegisterService_out {
    pub error: ErrorType,}
pub fn sam_RegisterService(req: &mut sam_request_RegisterService_in, req_arena: &MessageArena, resp: &mut sam_request_RegisterService_out, resp_arena: &mut MessageArena) -> Result<usize, usize> {

    let mut ipc = IpcMessage::new();

    ipc.set_out_arena(req_arena.as_slice_allocated());

    // if let Some(arena) = resp_arena {
        ipc.set_in_arena(resp_arena.as_slice());
    // }

    ipc.set_mid(12853408287206418855);

    req.h = ipc.add_handle(req.h);


    unsafe { SERVER_HANDLE.as_ref().unwrap().call(&mut ipc) }?;
    let h = ipc.handles();
let mut resp_ = resp_arena.read::<sam_request_RegisterService_out>(ArenaPtr::request_ptr::<sam_request_RegisterService_out>()).unwrap();
                     let error = resp_.error;
                     if error != 0.into() {
                        return Err(error.into());
                     }
                    
    *resp = resp_;

    Ok(0)
}
