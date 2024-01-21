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
    pub tmp: i32,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_FindService_out {
    pub tmp1: i32,}

pub struct ServerVirtTable {
    pub cb_FindService: fn(sam_request_FindService_in) -> sam_request_FindService_out,
}

pub fn start_server(cbs: ServerVirtTable, p: Port) -> ! {
    unsafe {
        SERVER_HANDLE = Some(p);
    }

    loop {
        unsafe {
            union Buffer___ {
                req_FindService: sam_request_FindService_in,

            };

            let mut buff = [0u8; core::mem::size_of::<Buffer___>() + core::mem::size_of::<RequestHeader>()];
            let header: *const RequestHeader = core::mem::transmute(buff.as_ptr());

            let port = SERVER_HANDLE.as_ref().unwrap().receive_data(bytemuck::bytes_of_mut(&mut buff)).unwrap().unwrap();

            let n = (*header).num;
            match (*header).num {
                
                    6790964161597629750 => {
                        let arg: *const sam_request_FindService_in = 
                            core::mem::transmute(
                                buff.as_ptr().offset(core::mem::size_of::<RequestHeader>() as isize)
                            );
                        let res = (cbs.cb_FindService)(*arg);
                        SERVER_HANDLE.as_ref().unwrap().send_data(port, bytemuck::bytes_of(&res));
                    }
                            
                _ => { panic!() }
            };
        }
    };
}

