#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(thread_local)]

use alloc::string::ToString;
use libc::main;
use libc::port::Port;
use libc::task::Task;
use ridlrt::arena::MessageArena;
use rtl::handle::Handle;
use rtl::handle::HANDLE_INVALID;
use rtl::error::ErrorType;
use rtl::cpio::Cpio;

mod interface;
use interface::*;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

fn handle_req(
    r: sam_request_FindService_in,
    req_arena: &MessageArena,
    resp_arena: &mut MessageArena,
) -> Result<sam_request_FindService_out, ErrorType> {
    let mut name_buf = [0u8; 100];
    let size = req_arena.read_slice(r.name, &mut name_buf).unwrap();
    let name = core::str::from_utf8(&name_buf[..size]).unwrap();

    let p = r.name;
    println!("CLIENT REQ: {:?}", p);
    println!("CLIENT REQ: {name}");

    Ok(sam_request_FindService_out {
        h: 10,
    })
}

#[main]
fn main(boot_handle: Handle) {
    println!("Init proccess started");

    assert!(boot_handle == HANDLE_INVALID);

    let cpio = Cpio::new(CPIO).unwrap();

    let p = Port::create().unwrap();

    for i in cpio.iter() {
        println!("{:?}", i);

        let elf = i.data();
        let mut task =
            Task::create_from_elf(elf, "test task".to_string()).expect("Failed to create task");
        task.start(p.handle()).unwrap();

        println!("Spawned '{}'", task.name())
    }

    let virt_table = Disp {
        cb_FindService: handle_req,
    };

    let info = ridlrt::server::ServerInfo {
        h: p.handle(),
        dispatch: virt_table,
    };

    ridlrt::server::server_dispatch(&info);
}
