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

    use ridlrt::server::Dispatcher;
    use ridlrt::arena::MessageArena;

    pub struct Disp {
        pub cb_FindService: fn(sam_request_FindService_in, req_arena: &MessageArena, resp_arena: &mut MessageArena) -> Result<sam_request_FindService_out, ErrorType>,

    }

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union RequestUnion {
        pub req_FindService: sam_request_FindService_in,

    }

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union ResponseUnion {
        pub req_FindService: sam_request_FindService_out,

    }

    impl Dispatcher for Disp {
        type DispatchReq = RequestUnion;
        type DispatchResp = ResponseUnion;

        fn dispatch(
            &self,
            mid: usize,
            request: &Self::DispatchReq,
            req_arena: &MessageArena,
            response: &mut Self::DispatchResp,
            resp_arena: &mut MessageArena,
        ) {
            match mid {
                
                    6790964161597629750 => {
                        let arg = unsafe { &request.req_FindService };

                        response.req_FindService = (self.cb_FindService)(*arg, req_arena, resp_arena).unwrap();
                    }
                            
                _ => panic!(),
            }
        }
    }
        
