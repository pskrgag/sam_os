use crate::client::serial::*;
use bytemuck::*;
use ridlrt::arena::*;
use rtl::handle::*;

pub fn init(h: Handle) {
    sam_transport_init(h);
}

pub fn read_byte() -> Result<u8, usize> {
    let mut resp_arena_buf = [0u8; 1000];
    let req_arena = MessageArena::new_backed(&mut []);
    let mut resp_arena = MessageArena::new_backed(&mut resp_arena_buf);
    let mut resp = sam_request_ReadByte_out::zeroed();

    let mut req = sam_request_ReadByte_in::zeroed();

    sam_ReadByte(&mut req, &req_arena, &mut resp, &mut resp_arena)?;

    Ok(resp.b)
}

pub fn write_byte(b: u8) -> Result<(), usize> {
    let mut resp_arena_buf = [0u8; 1200];
    let mut req_arena_buf = [0u8; 10];
    let mut req_arena = MessageArena::new_backed(&mut req_arena_buf);
    let mut resp_arena = MessageArena::new_backed(&mut resp_arena_buf);
    let mut resp = sam_request_WriteByte_out::zeroed();

    let req = req_arena
        .allocate::<sam_request_WriteByte_in>(&sam_request_WriteByte_in::zeroed())
        .unwrap();
    let mut req = req
        .ptr_to_native_in_arena::<sam_request_WriteByte_in>(&req_arena)
        .unwrap();

    req.b = b;

    sam_WriteByte(&mut req, &req_arena, &mut resp, &mut resp_arena)?;

    Ok(())
}

pub fn write_bytes(b: &[u8]) -> Result<(), usize> {
    let mut resp_arena_buf = [0u8; 1000];
    let mut req_arena_buf = [0u8; 1000];
    let mut req_arena = MessageArena::new_backed(&mut req_arena_buf);
    let mut resp_arena = MessageArena::new_backed(&mut resp_arena_buf);
    let mut resp = sam_request_WriteBytes_out::zeroed();

    let req = req_arena
        .allocate::<sam_request_WriteBytes_in>(&sam_request_WriteBytes_in::zeroed())
        .unwrap();
    let mut req = req
        .ptr_to_native_in_arena::<sam_request_WriteBytes_in>(&req_arena)
        .unwrap();

    req.str = req_arena.allocate_slice(b).unwrap();

    sam_WriteBytes(&mut req, &req_arena, &mut resp, &mut resp_arena)?;

    Ok(())
}
