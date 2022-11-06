use crate::mm::types::*;

#[derive(Debug)]
pub enum MmError {
    InvalidAddr,
    NotImpl,
}

#[derive(Clone, Copy)]
pub enum MappingType {
    KernelData,
    KernelText,
    KernelDataRo,
    KernelRWX,
    KernelDevice,
}

pub trait PageTable {
    /* If p is None then caller want linear mapping */
    fn map(
        &mut self,
        _p: Option<MemRange<PhysAddr>>,
        _v: MemRange<VirtAddr>,
        _m_type: MappingType,
    ) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    fn unmap(&mut self, _v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    fn base(&self) -> PhysAddr;
    fn entries_per_lvl(&self) -> usize;
}

pub trait TranslationTableBlock {
    type RawBlock;

    fn invalid() -> Self;
    fn set_mapping_type(&mut self, tp: MappingType);
    fn get(&self) -> Self::RawBlock;
    fn invalidate(&mut self);
    fn valid(&mut self);
    fn set_OA(&mut self, addr: PhysAddr);
}

pub trait TranslationTableTable {
    type RawTable;

    fn invalid() -> Self;
    fn valid(&mut self);
    fn get(&self) -> Self::RawTable;
    fn invalidate(&mut self);
    fn set_OA(&mut self, addr: PhysAddr);
}
