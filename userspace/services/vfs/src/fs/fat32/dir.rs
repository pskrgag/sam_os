use super::sb::{data_from_bytes, Cluster, SuperBlockRef};
use crate::fs::inode::Inode;
use alloc::vec;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

// On disk representation
#[repr(C)]
#[derive(Debug)]
pub struct FsDirEntry {
    name: [u8; 11], /* name and extension */
    attr: u8,       /* attribute bits */
    lcase: u8,      /* Case for base and extension */
    ctime_cs: u8,   /* Creation time, centiseconds (0-199) */
    ctime: u16,     /* Creation time */
    cdate: u16,     /* Creation date */
    adate: u16,     /* Last access date */
    starthi: u16,   /* High 16 bits of cluster in FAT32 */
    time: u16,
    date: u16,
    start: u16, /* time, date and first cluster */
    size: u32,  /* file size (in bytes) */
}

impl FsDirEntry {
    pub fn is_used(&self) -> bool {
        self.name[0] == 0 || self.name[0] == 0xe5
    }
}

pub struct Fat32Dir {
    sb: SuperBlockRef,
    start: Cluster,
}

impl Fat32Dir {
    pub async fn new(sb: SuperBlockRef, start: Cluster) -> Result<Self, ErrorType> {
        Ok(Self { sb, start })
    }
}

impl Inode for Fat32Dir {}
