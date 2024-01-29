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
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_RegisterService_in {
    pub name: ArenaPtr,    pub h: Handle,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_RegisterService_out {
    pub error: ErrorType,}

    use ridlrt::server::Dispatcher;
    use ridlrt::arena::MessageArena;

    pub struct Disp {
        pub cb_FindService: fn(sam_request_FindService_in, req_arena: &MessageArena, resp_arena: &mut MessageArena) -> Result<sam_request_FindService_out, ErrorType>,
pub cb_RegisterService: fn(sam_request_RegisterService_in, req_arena: &MessageArena, resp_arena: &mut MessageArena) -> Result<sam_request_RegisterService_out, ErrorType>,

    }

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union RequestUnion {
        pub req_FindService: sam_request_FindService_in,
pub req_RegisterService: sam_request_RegisterService_in,

    }

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union ResponseUnion {
        pub req_FindService: sam_request_FindService_out,
pub req_RegisterService: sam_request_RegisterService_out,

    }

    impl Dispatcher for Disp {
        type DispatchReq = RequestUnion;
        type DispatchResp = ResponseUnion;

        fn dispatch(
            &self,
            in_ipc: &IpcMessage,
            out_ipc: &mut IpcMessage,
            request: &mut Self::DispatchReq,
            req_arena: &MessageArena,
            response: &mut Self::DispatchResp,
            resp_arena: &mut MessageArena,
        ) {
            match in_ipc.mid() {
                
                    6790964161597629750 => {
                        let arg = unsafe { &mut request.req_FindService };

                        let h = in_ipc.handles();
;

                        match (self.cb_FindService)(*arg, req_arena, resp_arena) {
                            Ok(rr) => { 
                                response.req_FindService = rr;
                                response.req_FindService.h = out_ipc.add_handle(unsafe { response.req_FindService.h })
                            },
                            Err(err) => response.req_FindService.error = err,
                        };

                    }
                            
                    12853408287206418855 => {
                        let arg = unsafe { &mut request.req_RegisterService };

                        let h = in_ipc.handles();
    arg.h = h[0];
;

                        match (self.cb_RegisterService)(*arg, req_arena, resp_arena) {
                            Ok(rr) => { 
                                response.req_RegisterService = rr;
                                
                            },
                            Err(err) => response.req_RegisterService.error = err,
                        };

                    }
                            
                _ => panic!(),
            }
        }
    }
        
