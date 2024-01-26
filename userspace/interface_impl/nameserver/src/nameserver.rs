use interfaces::server::nameserver::*;
use libc::port::Port;
use ridlrt::arena::MessageArena;
use rtl::error::ErrorType;
use rtl::handle::Handle;

use alloc::collections::BTreeMap;
use alloc::string::*;
use rtl::locking::fake_lock::FakeLock;

static SERVICES: FakeLock<BTreeMap<String, Handle>> = FakeLock::new(BTreeMap::new());

fn find_servive(
    r: sam_request_FindService_in,
    req_arena: &MessageArena,
    _resp_arena: &mut MessageArena,
) -> Result<sam_request_FindService_out, ErrorType> {
    let mut name_buf = [0u8; 100];
    let size = req_arena.read_slice(r.name, &mut name_buf).unwrap();
    let name = core::str::from_utf8(&name_buf[..size]).unwrap();

    let h = SERVICES
        .get()
        .get(&name.to_string())
        .ok_or(ErrorType::INVALID_ARGUMENT)?;

    Ok(sam_request_FindService_out {
        error: 0.into(),
        h: *h,
    })
}

fn register_service(
    r: sam_request_RegisterService_in,
    req_arena: &MessageArena,
    _resp_arena: &mut MessageArena,
) -> Result<sam_request_RegisterService_out, ErrorType> {
    let mut name_buf = [0u8; 100];
    let size = req_arena.read_slice(r.name, &mut name_buf).unwrap();
    let name = core::str::from_utf8(&name_buf[..size]).unwrap();

    SERVICES.get().insert(name.to_string(), r.h);

    println!("Registered service '{name}'");
    Ok(sam_request_RegisterService_out { error: 0.into() })
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

    ridlrt::server::server_dispatch(&info).unwrap();
}
