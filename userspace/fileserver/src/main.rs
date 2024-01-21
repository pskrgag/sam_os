#![no_std]
#![no_main]
#![feature(format_args_nl)]

use libc::main;
use libc::port::Port;
use libc::vmm::vms::vms;
use rtl::arch::PAGE_SIZE;
use rtl::handle::{Handle, HANDLE_INVALID};
use rtl::uart::*;
use rtl::vmm::types::*;

// mod transport;
// use transport::*;

fn stupid_ipc_test(h: Handle) {
    use ridlrt::arena::MessageArena;
    use rtl::ipc::IpcMessage;

    let p = Port::new(h);
    let mut b = [0u8; 1000];
    let mut ipc = IpcMessage::new();
    let mut arena = MessageArena::new_backed(b.as_mut_slice());

    let ptr = arena.allocate_slice("hello, server".as_bytes()).unwrap();
    ipc.set_mid(1234);
    ipc.set_out_arena(arena.as_slice_allocated());

    println!("{:?} {:?}", ptr, ipc);
    p.call(&mut ipc);
}

#[main]
fn main(boot_handle: Handle) {
    assert!(boot_handle != HANDLE_INVALID);

    let base = vms()
        .map_phys(MemRange::<PhysAddr>::new(0x09000000.into(), PAGE_SIZE))
        .unwrap();

    let mut uart = Uart::init(base);
    let mut b = [1u8; 10];

    stupid_ipc_test(boot_handle);
}
