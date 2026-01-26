use super::Card;
use crate::regs::ResponseU128;
use crate::regs::SdhciError;
use crate::sdhci::SdhciIface;
use alloc::vec::Vec;
use bitvec::field::BitField;
use core::ops::Deref;
use rtl::error::ErrorType;

#[derive(Default)]
struct CSD {
    csd_structure: u8,
    device_size: usize,
}

impl From<ResponseU128> for CSD {
    fn from(value: ResponseU128) -> Self {
        let csd_structure: u8 = value.range(126..128).load_le();

        match csd_structure {
            0 => {
                let device_size: u32 = value.range(62..74).load_le();
                let dev_size_mul: u32 = value.range(47..50).load_le();
                let read_blk_len: u32 = value.range(80..84).load_le();

                let tmp_blk_count = (device_size + 1) << (dev_size_mul + 2);
                CSD {
                    csd_structure,
                    device_size: (tmp_blk_count * (1 << read_blk_len)) as _,
                }
            }
            1 => {
                panic!("TODO?");
            }
            _ => panic!("IDK"),
        }
    }
}

pub struct SDCard {
    iface: SdhciIface,
    csd: CSD,
    block_size: u16,
}

impl SDCard {
    pub fn new(mut iface: SdhciIface) -> Result<Self, SdhciError> {
        Ok(Self {
            csd: iface.csd()?.into(),
            iface,
            block_size: 512,
        })
    }
}

impl Deref for SDCard {
    type Target = SdhciIface;

    fn deref(&self) -> &Self::Target {
        &self.iface
    }
}

impl Card for SDCard {
    fn read_block(&mut self, block: u32) -> Result<Vec<u8>, ErrorType> {
        let block_size = self.block_size();

        self.iface
            .with_selected_card(|mut card| card.read_block(block, block_size))
    }

    fn write_block(&mut self, block: u32, data: &[u8]) -> Result<(), ErrorType> {
        let block_size = self.block_size();

        self.iface
            .with_selected_card(|mut card| card.write_block(block, block_size, data))
    }

    fn block_size(&self) -> u16 {
        self.block_size
    }

    fn device_size(&self) -> usize {
        self.csd.device_size
    }

    fn set_block_size(&mut self, block_size: u16) -> Result<(), ErrorType> {
        if !(block_size >= 512 && block_size <= 4096) {
            return Err(ErrorType::InvalidArgument);
        }

        self.block_size = block_size;
        Ok(())
    }
}
