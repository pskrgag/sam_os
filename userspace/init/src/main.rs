#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(thread_local)]

use alloc::string::ToString;
use libc::main;
use libc::port::Port;
use libc::task::Task;
use rtl::handle::Handle;
use rtl::handle::HANDLE_INVALID;

use rtl::cpio::Cpio;

// mod interface;
// use interface::*;

static CPIO: &[u8] = include_bytes!("/tmp/archive.cpio");

// fn handle_req(r: sam_request_FindService_in) -> sam_request_FindService_out {
//     let n = r.tmp;
//
//     sam_request_FindService_out {
//         tmp1: 1337,
//     }
// }

fn stupid_ipc_test(h: Handle) {
    use ridlrt::arena::*;
    use rtl::ipc::IpcMessage;

    let mut p = Port::new(h);
    let mut b = [0u8; 100];
    let mut s = [0u8; 100];
    let mut ipc = IpcMessage::new();
    let mut arena = MessageArena::new_backed(b.as_mut_slice());

    ipc.set_in_arena(arena.as_slice());

    p.receive_data(&mut ipc).unwrap();

    let size = arena
        .read_slice(ArenaPtr {
            offset: 0,
            size: 13,
            _p: core::marker::PhantomData::<u8>,
        }, &mut s)
        .unwrap();

    println!("{:?}", core::str::from_utf8(&s[..size]).unwrap());
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

    stupid_ipc_test(p.handle());

    // let virt_table = ServerVirtTable {
    //     cb_FindService: handle_req,
    // };
    //
    // start_server(virt_table, p);
}
