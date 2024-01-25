use crate::interface::*;
use libc::port::Port;
use ridlrt::arena::MessageArena;
use rtl::error::ErrorType;
use rtl::handle::Handle;

use rtl::locking::fake_lock::FakeLock;
use alloc::collections::BTreeMap;
use alloc::string::*;

static SERVICES: FakeLock<BTreeMap<String, Handle>> = FakeLock::new(BTreeMap::new());

fn find_servive(
    r: sam_request_FindService_in,
    req_arena: &MessageArena,
    resp_arena: &mut MessageArena,
) -> Result<sam_request_FindService_out, ErrorType> {
    let mut name_buf = [0u8; 100];
    let size = req_arena.read_slice(r.name, &mut name_buf).unwrap();
    let name = core::str::from_utf8(&name_buf[..size]).unwrap();

    println!("CLIENT REQ: {name}");

    Ok(sam_request_FindService_out { h: 10 })
}

fn register_service(
    r: sam_request_RegisterService_in,
    req_arena: &MessageArena,
    resp_arena: &mut MessageArena,
) -> Result<sam_request_RegisterService_out, ErrorType> {
    let mut name_buf = [0u8; 100];
    let size = req_arena.read_slice(r.name, &mut name_buf).unwrap();
    assert!(size < 100);
    let name = core::str::from_utf8(&name_buf[..size]).unwrap();

    let h = r.h;
    println!("CLIENT REQ: {name} {}", h);

    SERVICES.get().insert(name.to_string(), r.h);

    Ok(sam_request_RegisterService_out {})
}

pub fn start_nameserver(p: Port) {
    let virt_table = Disp {
        cb_FindService: find_servive,
        cb_RegisterService: register_service,
    };

    let info = ridlrt::server::ServerInfo {
        h: p.handle(),
        dispatch: virt_table,
    };

    ridlrt::server::server_dispatch(&info);
}
