use libc::port::Port;
use rtl::handle::*;
use bytemuck;
use bytemuck::Zeroable;

static mut SERVER_HANDLE: Option<Port> = None;

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
struct RequestHeader {
    pub num: u64,
}

pub fn init(h: Handle) {
    if h != HANDLE_INVALID {
        unsafe { SERVER_HANDLE = Some(Port::new(h)); }
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_FindService_in {
    pub hdr: RequestHeader,    pub tmp: i32,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_FindService_out {
    pub tmp1: i32,}
pub fn FindService(tmp: &i32, tmp1: &mut i32) -> Result<usize, usize> {
    let to_call_in = sam_request_FindService_in {
        hdr: RequestHeader { num: 6790964161597629750 },
        tmp: *tmp,
    };
    let mut to_call_out = sam_request_FindService_out::zeroed();
    unsafe {
        let r = if core::mem::size_of_val(&to_call_out) != 0 { Some(bytemuck::bytes_of_mut(&mut to_call_out)) } else { None };
        SERVER_HANDLE.as_ref().unwrap().call(bytemuck::bytes_of(&to_call_in), r);
        }
*tmp1 = to_call_out.tmp1;
    Ok(0)
}
