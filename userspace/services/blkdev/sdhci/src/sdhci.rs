use super::cards::sd::SDCard;
use super::cards::Card;
use super::regs::response::*;
use super::regs::SdhciError;
use super::regs::SdhciIrq;
use super::regs::{ApplicationOpcode, Command, CommandFlag, NormalOpcode, ResponseU128};
use alloc::boxed::Box;
use alloc::vec::Vec;
use bitvec::field::BitField;
use core::ptr::NonNull;
use hal::address::{VirtAddr, VirtualAddress};
use rtl::error::ErrorType;
use safe_mmio::{
    field, field_shared,
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
#[allow(dead_code)]
pub struct SdhciVersion {
    /// Specification Version Number
    pub sdhc_version: u8,
    /// Vendor Version Number
    pub vendor: u8,
}

#[allow(dead_code)]
enum AddressingMode {
    Byte,
    Block,
}

/// SDHCI interface
pub struct SdhciIface {
    regs: UniqueMmioPointer<'static, SdhciRegs>,
    rca: u16,
    mode: AddressingMode,
}

pub struct SelectedCard<'a> {
    sdhci: &'a mut SdhciIface,
}

fn spin_retry<F: FnMut() -> bool>(mut f: F, retries: usize) -> bool {
    let mut retries = retries - 1;

    while !f() && retries != 0 {
        retries -= 1;
        core::hint::spin_loop();
    }

    f()
}

impl SelectedCard<'_> {
    pub fn write_block(
        &mut self,
        block_address: u32,
        block_size: u16,
        data: &[u8],
    ) -> Result<(), ErrorType> {
        // Setup block size
        field!(self.sdhci.regs, block_size).write(block_size);

        // Read one block
        field!(self.sdhci.regs, block_count).write(1);

        if field!(self.sdhci.regs, block_size).read() as u16 != block_size {
            return Err(ErrorType::InvalidArgument);
        }

        // Set direction to write
        field!(self.sdhci.regs, transfer_mode).write(0x0);

        let addr = match self.sdhci.mode {
            AddressingMode::Byte => block_address * block_size as u32,
            AddressingMode::Block => block_address,
        };

        // Issue block write
        self.sdhci
            .send_command::<ResponseU128>(Command::new_normal(
                NormalOpcode::WriteOneBlock,
                addr,
                CommandFlag::HasReponse | CommandFlag::DataPresent,
            ))?;

        for chunk in data.chunks(core::mem::size_of::<u32>()) {
            field!(self.sdhci.regs, buffer_data)
                .write(u32::from_ne_bytes(chunk.try_into().unwrap()));
        }

        Ok(())
    }

    pub fn read_block(
        &mut self,
        block_address: u32,
        block_size: u16,
    ) -> Result<Vec<u8>, ErrorType> {
        let mut data = Vec::new();

        // Setup block size
        field!(self.sdhci.regs, block_size).write(block_size);

        if field!(self.sdhci.regs, block_size).read() as u16 != block_size {
            return Err(ErrorType::InvalidArgument);
        }

        data.resize(block_size as _, 0);

        // Set direction to write
        field!(self.sdhci.regs, transfer_mode).write(0x0010);

        let addr = match self.sdhci.mode {
            AddressingMode::Byte => block_address * block_size as u32,
            AddressingMode::Block => block_address,
        };

        // Issue block read
        self.sdhci
            .send_command::<ResponseU128>(Command::new_normal(
                NormalOpcode::ReadOneBlock,
                addr,
                CommandFlag::HasReponse | CommandFlag::DataPresent,
            ))?;

        for chunk in data.chunks_mut(core::mem::size_of::<u32>()) {
            let ch = field!(self.sdhci.regs, buffer_data).read();

            chunk.copy_from_slice(&ch.to_ne_bytes());
        }

        Ok(data)
    }
}

impl SdhciIface {
    /// Retrieves SDHCI version.
    pub fn version(&self) -> SdhciVersion {
        // SAFETY: reading version does not causes side effects.
        let raw = unsafe { field_shared!(self.regs, version).read_unsafe() }.0;

        SdhciVersion {
            sdhc_version: (raw & 0xFF) as _,
            vendor: (raw >> 8) as _,
        }
    }

    // Calls callback with current card selected
    pub fn with_selected_card<T, F: FnMut(SelectedCard) -> Result<T, ErrorType>>(
        &mut self,
        mut f: F,
    ) -> Result<T, ErrorType> {
        // Select current card
        self.send_command::<ResponseU32>(Command::new_normal(
            NormalOpcode::SelectCard,
            (self.rca as u32) << 16,
            CommandFlag::HasReponse,
        ))?;

        let res = f(SelectedCard { sdhci: self });

        // Set 0 as RCA to deselect card
        self.send_command(Command::<ResponseU32>::new_normal(
            NormalOpcode::SelectCard,
            0,
            CommandFlag::HasReponse,
        ))?;

        res
    }

    /// Initializes the hw
    fn setup(regs: UniqueMmioPointer<'static, SdhciRegs>) -> Result<Box<dyn Card>, ErrorType> {
        let mut card = Self {
            regs,
            rca: 0,
            mode: AddressingMode::Byte,
        };

        // Enable clocks: Enable internal clock and SD clock
        field!(card.regs, clock_on).write(0b101);

        // Wait for clocks to stabilize
        if !spin_retry(|| field!(card.regs, clock_on).read() & 0b01 != 0, 100) {
            return Err(ErrorType::TryAgain);
        }

        // Send CMD0 (reset)
        let cmd0 = Command::<NoResponse>::new_normal(
            NormalOpcode::GoIdleState,
            0,
            CommandFlag::NoResponse,
        );
        card.send_command(cmd0)?;

        // Send CMD8 (check voltage)
        // 1 is 2.7-3.6V
        let cmd8 = Command::<ResponseU32>::new_normal(
            NormalOpcode::VoltageCheck,
            (1 << 16) | 0xAA,
            CommandFlag::HasReponse,
        );
        // TODO: verify check pattern
        let _f8 = card.send_command(cmd8)?.is_some();

        // CMD5 is not implemented in QEMU. Assuming !SDIO

        // Send ACMD41
        let acmd41 = Command::<ResponseU32>::new_application(
            ApplicationOpcode::OpCond,
            0x40020000,
            CommandFlag::HasReponse,
        );

        if let Some(resp) = card.send_command(acmd41)? {
            if resp.range(30..31).first().unwrap() == false {
                // It is SDSC.

                // This will advance card to standby state
                let _cid = card
                    .send_command(Command::<ResponseU32>::new_normal(
                        NormalOpcode::CID,
                        0,
                        CommandFlag::HasReponse,
                    ))?
                    .unwrap();

                // Get relative address.
                let rca = card
                    .send_command(Command::<ResponseU32>::new_normal(
                        NormalOpcode::RCA,
                        0,
                        CommandFlag::HasReponse,
                    ))?
                    .unwrap();

                card.rca = rca.range(16..32).load();
                return Ok(Box::new(SDCard::new(card)?));
            } else {
                // If it's zero then it is SDHC
                todo!("{:?}", resp.range(31..32));
            }
        }

        todo!()
    }

    pub fn csd(&mut self) -> Result<ResponseU128, SdhciError> {
        Ok(self
            .send_command(Command::<ResponseU128>::new_normal(
                NormalOpcode::CSD,
                (self.rca as u32) << 16,
                CommandFlag::HasReponse,
            ))?
            .unwrap())
    }

    fn send_command<R: Response>(&mut self, cmd: Command<R>) -> Result<Option<R>, SdhciError> {
        if cmd.is_application() {
            let app = Command::<ResponseU32>::new_normal(
                NormalOpcode::AppCmd,
                0,
                CommandFlag::HasReponse,
            );

            self.send_one_command(app)?;
            self.send_one_command(cmd)
        } else {
            self.send_one_command(cmd)
        }
    }

    fn send_one_command<R: Response>(&mut self, cmd: Command<R>) -> Result<Option<R>, SdhciError> {
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
        if !spin_retry(
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

pub fn probe(base: VirtAddr) -> Result<Box<dyn Card>, ErrorType> {
    debug_assert!(!base.is_null());

    SdhciIface::setup(unsafe { UniqueMmioPointer::new(NonNull::new_unchecked(base.to_raw_mut())) })
}
