use crate::rt::nameserver::*;
use bytemuck::*;
use ridlrt::arena::*;
use rtl::handle::*;

pub fn init(h: Handle) {
    sam_transport_init(h);
}

pub fn FindService(name: &str) -> Result<Handle, usize> {
    let mut req_arena_buf = [0u8; 100];
    let mut req_arena = MessageArena::new_backed(&mut req_arena_buf);
    let mut resp = sam_request_FindService_out::zeroed();

    let req_ptr =
        req_arena.allocate::<sam_request_FindService_in>(&sam_request_FindService_in::zeroed()).unwrap();
    let req = req_ptr.ptr_to_native_in_arena::<sam_request_FindService_in>(&req_arena).unwrap();

    req.name = req_arena.allocate_slice(name.as_bytes()).unwrap();

    let p = req.name;
    sam_FindService(&req, &req_arena, &mut resp, None).unwrap();

    todo!()
}
