use interfaces::server::serial::*;
use libc::port::Port;
use libc::vmm::vms::vms;
use ridlrt::arena::MessageArena;
use rtl::arch::PAGE_SIZE;
use rtl::error::ErrorType;
use rtl::locking::fake_lock::FakeLock;
use rtl::uart::*;
use rtl::vmm::types::*;

static UART: FakeLock<Uart> = FakeLock::new(Uart::default(VirtAddr::new(0x0)));

fn get_byte(
    _r: sam_request_ReadByte_in,
    _req_arena: &MessageArena,
    _resp_arena: &mut MessageArena,
) -> Result<sam_request_ReadByte_out, ErrorType> {
    let mut n = [0u8; 1];

    UART.get().read_bytes(&mut n);
    Ok(sam_request_ReadByte_out {
        error: 0.into(),
        b: n[0],
    })
}

fn put_byte(
    r: sam_request_WriteByte_in,
    _req_arena: &MessageArena,
    _resp_arena: &mut MessageArena,
) -> Result<sam_request_WriteByte_out, ErrorType> {
    UART.get().write_bytes(&[r.b]);
    Ok(sam_request_WriteByte_out { error: 0.into() })
}

fn put_bytes(
    r: sam_request_WriteBytes_in,
    req_arena: &MessageArena,
    _resp_arena: &mut MessageArena,
) -> Result<sam_request_WriteBytes_out, ErrorType> {
    let mut buf = [0u8; 100];
    let size = req_arena.read_slice(r.str, &mut buf).unwrap();

    UART.get().write_bytes(&buf[..size]);
    Ok(sam_request_WriteBytes_out { error: 0.into() })
}

pub fn start_serial(p: Port) {
    let virt_table = Disp {
        cb_ReadByte: get_byte,
        cb_WriteByte: put_byte,
        cb_WriteBytes: put_bytes,
    };

    let info = ridlrt::server::ServerInfo {
        h: p.handle(),
        dispatch: virt_table,
    };

    let base = vms()
        .map_phys(MemRange::<PhysAddr>::new(0x09000000.into(), PAGE_SIZE))
        .unwrap();

    *UART.get() = Uart::init(base);

    ridlrt::server::server_dispatch(&info).unwrap();
}
