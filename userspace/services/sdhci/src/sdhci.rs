use super::regs::SdhciError;
use super::regs::SdhciIrq;
use super::regs::{ApplicationOpcode, Command, CommandFlag, NormalOpcode, Response};
use core::ptr::NonNull;
use hal::address::{VirtAddr, VirtualAddress};
use rtl::error::ErrorType;
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
    argument: ReadWrite<u32>,
    transfer_mode: ReadWrite<u16>,
    cmdreg: ReadWrite<u16>,
    response: ReadOnly<[u32; 4]>,
    buffer_data: ReadWrite<u32>,
    present_state: ReadOnly<u32>,
    hostctl: ReadWrite<u8>,
    power_ctrl: ReadWrite<u8>,
    block_gap: ReadWrite<u8>,
    wake_up: ReadWrite<u8>,
    clock_on: ReadWrite<u16>,
    timeout: ReadWrite<u8>,
    reset: ReadWrite<u8>,
    normal_interrupt_status: ReadOnly<u16>,
    error_interrupt_status: ReadOnly<u16>,
    normal_interrupt_enable: ReadWrite<u16>,
    error_interrupt_enable: ReadWrite<u16>,
    normal_signal_enable: ReadWrite<u16>,
    error_signal_enable: ReadWrite<u16>,
    acmd12_status: ReadWrite<u16>,
    hostctl2: ReadWrite<u16>,
    caps: ReadWrite<u64>,
    cur_caps: ReadWrite<u64>,
    feaer: ReadWrite<u16>,
    feerr: ReadWrite<u16>,
    dma_err: ReadWrite<u32>,
    dma_sys_addr: ReadWrite<u32>,
    _empty: [u8; 160],
    slot_int_status: ReadWrite<u16>,
    version: ReadWrite<u16>,
}

/// Host Controller Register
#[derive(Debug, Copy, Clone)]
pub struct SdhciVersion {
    /// Specification Version Number
    pub sdhc_version: u8,
    /// Vendor Version Number
    pub vendor: u8,
}

/// SDHCI driver
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

    fn spin_retry<F: FnMut() -> bool>(mut f: F, retries: usize) -> bool {
        let mut retries = retries - 1;

        while !f() && retries != 0 {
            retries -= 1;
            core::hint::spin_loop();
        }

        f()
    }

    pub fn setup(&mut self) -> Result<(), ErrorType> {
        // Enable clocks: Enable internal clock and SD clock
        field!(self.regs, clock_on).write(0b101);

        // Wait for clocks to stabilize
        if !Self::spin_retry(|| field!(self.regs, clock_on).read() & 0b01 != 0, 100) {
            return Err(ErrorType::TryAgain);
        }

        // Send CMD0 (reset)
        let cmd0 =
            Command::new_normal(NormalOpcode::GoIdleState, 0, CommandFlag::NoResponse.into());
        self.send_command(cmd0)?;

        // Send CMD8 (check voltage)
        // 1 is 2.7-3.6V
        let cmd8 = Command::new_normal(
            NormalOpcode::VoltageCheck,
            (1 << 16) | 0xAA,
            CommandFlag::HasReponse.into(),
        );
        // TODO: verify check pattern
        let f8 = self.send_command(cmd8)?.is_some();

        // CMD5 is not implemented in QEMU. Assuming !SDIO

        // Send ACMD41
        while {
            let acmd41 = Command::new_application(
                ApplicationOpcode::OpCond,
                0x40020000,
                CommandFlag::HasReponse.into(),
            );
            let resp = self.send_command(acmd41)?.unwrap();

            // Loop while it is not ready
            resp.check_bit(31) == false
        } {}

        // Get CID
        let cmd2 = Command::new_normal(NormalOpcode::CID, 0, CommandFlag::HasReponse.into());
        let _cid = self.send_command(cmd2)?.is_some();

        // Get relative address
        let cmd3 = Command::new_normal(NormalOpcode::RCA, 0, CommandFlag::HasReponse.into());
        let _rca = self.send_command(cmd3)?.is_some();

        Ok(())
    }

    /// Retrieves SDHCI version.
    pub fn version(&mut self) -> SdhciVersion {
        let raw = field!(self.regs, version).read();

        SdhciVersion {
            sdhc_version: (raw & 0xFF) as _,
            vendor: (raw >> 8) as _,
        }
    }

    fn send_command(&mut self, cmd: Command) -> Result<Option<Response>, SdhciError> {
        if cmd.is_application() {
            let app = Command::new_normal(NormalOpcode::AppCmd, 0, CommandFlag::HasReponse.into());

            self.send_one_command(app)?;
            self.send_one_command(cmd)
        } else {
            self.send_one_command(cmd)
        }
    }

    fn send_one_command(&mut self, cmd: Command) -> Result<Option<Response>, SdhciError> {
        // Setup error registers
        field!(self.regs, error_interrupt_enable)
            .write(*(SdhciError::Timeout | SdhciError::CommandIndex));

        // Setup normal IRQ
        field!(self.regs, normal_interrupt_enable).write(*SdhciIrq::CommandCompleted);

        // Set argument
        field!(self.regs, argument).write(cmd.arg());

        let need_response = cmd.need_response();
        let raw_command: u16 = cmd.into();

        field!(self.regs, cmdreg).write(raw_command);

        // Wait until operation is completed
        if !Self::spin_retry(
            || field!(self.regs, normal_interrupt_status).read() & *SdhciIrq::CommandCompleted != 0,
            100,
        ) {
            return Err(SdhciError::Timeout);
        }

        let errors = field!(self.regs, error_interrupt_status).read();

        if errors == 0 {
            Ok(need_response.then_some(field!(self.regs, response).read().into()))
        } else {
            Err(unsafe { core::mem::transmute(errors) })
        }
    }
}
