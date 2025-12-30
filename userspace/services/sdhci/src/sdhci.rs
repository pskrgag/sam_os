use core::ptr::NonNull;
use hal::address::{VirtAddr, VirtualAddress};
use safe_mmio::{
    field,
    fields::{ReadOnly, ReadWrite},
    UniqueMmioPointer,
};

#[repr(C, packed)]
struct SdhciRegs {
    sdmasysad: ReadWrite<u32>,
    block_size: ReadWrite<u16>,
    block_count: ReadWrite<u16>,
    transfer_mode: ReadWrite<u16>,
    cmdreg: ReadWrite<u32>,
    response: ReadOnly<[u32; 4]>,
    buffer_data: ReadWrite<u32>,
    present_state: ReadOnly<u32>,
    hostctl: ReadWrite<u8>,
    power_ctrl: ReadWrite<u8>,
    block_gap: ReadWrite<u8>,
    wake_up: ReadWrite<u8>,
    clock_on: ReadWrite<u8>,
    timeout: ReadWrite<u8>,
    reset: ReadWrite<u8>,
    normal_interrupt_status: ReadOnly<u16>,
    error_interrupt_status: ReadOnly<u16>,
    normal_interrupt_enable: ReadWrite<u16>,
    error_interrupt_enable: ReadWrite<u16>,
    normal_signal_enable: ReadWrite<u16>,
    error_signal_enable: ReadWrite<u16>,
}

pub struct Sdhci {
    regs: UniqueMmioPointer<'static, SdhciRegs>,
}

impl Sdhci {
    pub fn new(base: VirtAddr) -> Self {
        debug_assert!(!base.is_null());
        Self {
            regs: unsafe { UniqueMmioPointer::new(NonNull::new_unchecked(base.to_raw_mut())) },
        }
    }

    pub fn block_size(&mut self) -> u16 {
        field!(self.regs, block_size).read()
    }
}
