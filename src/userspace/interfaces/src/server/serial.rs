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
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteByte_in {
    pub b: u8,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteByte_out {
    pub error: ErrorType,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteBytes_in {
    pub str: ArenaPtr,}
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C, packed)]#[allow(private_interfaces)]
pub struct sam_request_WriteBytes_out {
    pub error: ErrorType,}

    use ridlrt::server::Dispatcher;
    use ridlrt::arena::MessageArena;

    pub struct Disp {
        pub cb_ReadByte: fn(sam_request_ReadByte_in, req_arena: &MessageArena, resp_arena: &mut MessageArena) -> Result<sam_request_ReadByte_out, ErrorType>,
pub cb_WriteByte: fn(sam_request_WriteByte_in, req_arena: &MessageArena, resp_arena: &mut MessageArena) -> Result<sam_request_WriteByte_out, ErrorType>,
pub cb_WriteBytes: fn(sam_request_WriteBytes_in, req_arena: &MessageArena, resp_arena: &mut MessageArena) -> Result<sam_request_WriteBytes_out, ErrorType>,

    }

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union RequestUnion {
        pub req_ReadByte: sam_request_ReadByte_in,
pub req_WriteByte: sam_request_WriteByte_in,
pub req_WriteBytes: sam_request_WriteBytes_in,

    }

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union ResponseUnion {
        pub req_ReadByte: sam_request_ReadByte_out,
pub req_WriteByte: sam_request_WriteByte_out,
pub req_WriteBytes: sam_request_WriteBytes_out,

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
                
                    3547374036322012030 => {
                        let arg = unsafe { &mut request.req_ReadByte };

                        let h = in_ipc.handles();
;

                        match (self.cb_ReadByte)(*arg, req_arena, resp_arena) {
                            Ok(rr) => { 
                                response.req_ReadByte = rr;
                                
                            },
                            Err(err) => response.req_ReadByte.error = err,
                        };

                    }
                            
                    17840268626156070287 => {
                        let arg = unsafe { &mut request.req_WriteByte };

                        let h = in_ipc.handles();
;

                        match (self.cb_WriteByte)(*arg, req_arena, resp_arena) {
                            Ok(rr) => { 
                                response.req_WriteByte = rr;
                                
                            },
                            Err(err) => response.req_WriteByte.error = err,
                        };

                    }
                            
                    8925475826264573456 => {
                        let arg = unsafe { &mut request.req_WriteBytes };

                        let h = in_ipc.handles();
;

                        match (self.cb_WriteBytes)(*arg, req_arena, resp_arena) {
                            Ok(rr) => { 
                                response.req_WriteBytes = rr;
                                
                            },
                            Err(err) => response.req_WriteBytes.error = err,
                        };

                    }
                            
                _ => panic!(),
            }
        }
    }
        
