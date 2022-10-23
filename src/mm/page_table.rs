use crate::mm::types::*;

pub enum MmError {
    InvalidAddr,
    NotImpl
}

pub trait PageTable {
    fn map(&self, _p: MemRange<PhysAddr>, _v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    fn unmap(&self, _v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    fn lvl1(&self) -> VirtAddr;
    fn entries_per_lvl(&self) -> usize;
}
