use crate::client::nameserver::*;
use bytemuck::*;
use ridlrt::arena::*;
use rtl::error::*;
use rtl::handle::*;

pub fn init(h: Handle) {
    sam_transport_init(h);
}

pub fn find_service(name: &str) -> Result<Handle, ErrorType> {
    let mut req_arena_buf = [0u8; 100];
    let mut resp_arena_buf = [0u8; 100];
    let mut req_arena = MessageArena::new_backed(&mut req_arena_buf);
    let mut resp_arena = MessageArena::new_backed(&mut resp_arena_buf);
    let mut resp = sam_request_FindService_out::zeroed();

    let req = req_arena
        .allocate::<sam_request_FindService_in>(&sam_request_FindService_in::zeroed())
        .unwrap();
    let mut req = req
        .ptr_to_native_in_arena::<sam_request_FindService_in>(&req_arena)
        .unwrap();

    req.name = req_arena.allocate_slice(name.as_bytes()).unwrap();

    sam_FindService(&mut req, &req_arena, &mut resp, &mut resp_arena)?;

    let error = resp.error;
    if error != 0.into() {
        Err(resp.error)
    } else {
        Ok(resp.h)
    }
}

pub fn register_service(name: &str, h: Handle) -> Result<(), usize> {
    let mut req_arena_buf = [0u8; 100];
    let mut resp_arena_buf = [0u8; 100];
    let mut req_arena = MessageArena::new_backed(&mut req_arena_buf);
    let mut resp_arena = MessageArena::new_backed(&mut resp_arena_buf);
    let mut resp = sam_request_RegisterService_out::zeroed();

    let req = req_arena
        .allocate::<sam_request_RegisterService_in>(&sam_request_RegisterService_in::zeroed())
        .unwrap();
    let mut req = req
        .ptr_to_native_in_arena::<sam_request_RegisterService_in>(&req_arena)
        .unwrap();

    req.h = h;
    req.name = req_arena.allocate_slice(name.as_bytes()).unwrap();

    sam_RegisterService(&mut req, &req_arena, &mut resp, &mut resp_arena)?;

    Ok(())
}
