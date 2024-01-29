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
pub struct sam_request_ReadByte_in {
}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_ReadByte_out {
    pub error: ErrorType,    pub b: u8,}
pub fn sam_ReadByte(req: &mut sam_request_ReadByte_in, req_arena: &MessageArena, resp: &mut sam_request_ReadByte_out, resp_arena: &mut MessageArena) -> Result<usize, usize> {

    let mut ipc = IpcMessage::new();

    ipc.set_out_arena(req_arena.as_slice_allocated());

    // if let Some(arena) = resp_arena {
        ipc.set_in_arena(resp_arena.as_slice());
    // }

    ipc.set_mid(3547374036322012030);

    

    unsafe { SERVER_HANDLE.as_ref().unwrap().call(&mut ipc) }?;
    let h = ipc.handles();
let mut resp_ = resp_arena.read::<sam_request_ReadByte_out>(ArenaPtr::request_ptr::<sam_request_ReadByte_out>()).unwrap();
                     let error = resp_.error;
                     if error != 0.into() {
                        return Err(error.into());
                     }
                    
    *resp = resp_;

    Ok(0)
}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteByte_in {
    pub b: u8,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteByte_out {
    pub error: ErrorType,}
pub fn sam_WriteByte(req: &mut sam_request_WriteByte_in, req_arena: &MessageArena, resp: &mut sam_request_WriteByte_out, resp_arena: &mut MessageArena) -> Result<usize, usize> {

    let mut ipc = IpcMessage::new();

    ipc.set_out_arena(req_arena.as_slice_allocated());

    // if let Some(arena) = resp_arena {
        ipc.set_in_arena(resp_arena.as_slice());
    // }

    ipc.set_mid(17840268626156070287);

    

    unsafe { SERVER_HANDLE.as_ref().unwrap().call(&mut ipc) }?;
    let h = ipc.handles();
let mut resp_ = resp_arena.read::<sam_request_WriteByte_out>(ArenaPtr::request_ptr::<sam_request_WriteByte_out>()).unwrap();
                     let error = resp_.error;
                     if error != 0.into() {
                        return Err(error.into());
                     }
                    
    *resp = resp_;

    Ok(0)
}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteBytes_in {
    pub str: ArenaPtr,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteBytes_out {
    pub error: ErrorType,}
pub fn sam_WriteBytes(req: &mut sam_request_WriteBytes_in, req_arena: &MessageArena, resp: &mut sam_request_WriteBytes_out, resp_arena: &mut MessageArena) -> Result<usize, usize> {

    let mut ipc = IpcMessage::new();

    ipc.set_out_arena(req_arena.as_slice_allocated());

    // if let Some(arena) = resp_arena {
        ipc.set_in_arena(resp_arena.as_slice());
    // }

    ipc.set_mid(8925475826264573456);

    

    unsafe { SERVER_HANDLE.as_ref().unwrap().call(&mut ipc) }?;
    let h = ipc.handles();
let mut resp_ = resp_arena.read::<sam_request_WriteBytes_out>(ArenaPtr::request_ptr::<sam_request_WriteBytes_out>()).unwrap();
                     let error = resp_.error;
                     if error != 0.into() {
                        return Err(error.into());
                     }
                    
    *resp = resp_;

    Ok(0)
}
