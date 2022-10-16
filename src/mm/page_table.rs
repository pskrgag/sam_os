use crate::mm::types::*;

pub enum MmError {
    InvalidAddr,
    NotImpl
}

pub trait PageTable {
    fn map(&self, p: MemRange<PhysAddr>, v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    fn unmap(&self, v: MemRange<VirtAddr>) -> Result<(), MmError> {
        Err(MmError::NotImpl)
    }

    /* 1lvl is required */
    fn lvl1(&self) -> VirtAddr;
    fn entries_per_lvl(&self) -> usize;

    fn lvl2(&self) -> Option<VirtAddr> {
        None
    }

    fn lvl3(&self) -> Option<VirtAddr> {
        None
    }

    fn lvl4(&self) -> Option<VirtAddr> {
        None
    }
}

fn dump_lvl(ptr: Option<VirtAddr>, size: usize) {
     if ptr.is_none() {
        return;
     }

    let mut idx = 0;
    let ptr = ptr.unwrap();
    let slice: &[u64] = unsafe { core::slice::from_raw_parts(ptr.to_raw(), size) };

    println!("base addr 0x{:x}", ptr.get());

    for i in slice {
        println!("{}        0x{:x}", idx, i);
        idx += 1
    }

}

pub fn dump_page_table<T: PageTable>(table: &T) {
    let lvl1 = table.lvl1();
    let lvl2 = table.lvl2();
    let lvl3 = table.lvl3();
    let lvl4 = table.lvl4();

    dump_lvl(Some(lvl1), table.entries_per_lvl());
    dump_lvl(lvl2, table.entries_per_lvl());
    dump_lvl(lvl3, table.entries_per_lvl());
    dump_lvl(lvl4, table.entries_per_lvl());
}
