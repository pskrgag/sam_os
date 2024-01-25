use crate::rt::nameserver::*;
use bytemuck::*;
use ridlrt::arena::*;
use rtl::handle::*;

pub fn init(h: Handle) {
    sam_transport_init(h);
}

pub fn FindService(name: &str) -> Result<Handle, usize> {
    let mut req_arena_buf = [0u8; 100];
    let mut resp_arena_buf = [0u8; 100];
    let mut req_arena = MessageArena::new_backed(&mut req_arena_buf);
    let mut resp_arena = MessageArena::new_backed(&mut resp_arena_buf);
    let mut resp = sam_request_FindService_out::zeroed();

    let req = req_arena
        .allocate::<sam_request_FindService_in>(&sam_request_FindService_in::zeroed())
        .unwrap();
    let req = req
        .ptr_to_native_in_arena::<sam_request_FindService_in>(&req_arena)
        .unwrap();

    req.name = req_arena.allocate_slice(name.as_bytes()).unwrap();

    sam_FindService(&req, &req_arena, &mut resp, &mut resp_arena).unwrap();

    Ok(resp.h)
}

pub fn RegisterService(name: &str, h: Handle) -> Result<(), usize> {
    let mut req_arena_buf = [0u8; 100];
    let mut resp_arena_buf = [0u8; 100];
    let mut req_arena = MessageArena::new_backed(&mut req_arena_buf);
    let mut resp_arena = MessageArena::new_backed(&mut resp_arena_buf);
    let mut resp = sam_request_RegisterService_out::zeroed();

    let req = req_arena
        .allocate::<sam_request_RegisterService_in>(&sam_request_RegisterService_in::zeroed())
        .unwrap();
    let req = req
        .ptr_to_native_in_arena::<sam_request_RegisterService_in>(&req_arena)
        .unwrap();

    req.h = h;
    req.name = req_arena.allocate_slice(name.as_bytes()).unwrap();

    sam_RegisterService(&req, &req_arena, &mut resp, &mut resp_arena).unwrap();

    Ok(())
}
