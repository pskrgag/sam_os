use super::sb::{data_from_bytes, Cluster, SuperBlockRef};
use crate::bindings_Vfs::{DirEntry, DirEntryFlagsFlag};
use crate::fs::inode::{DirectoryOperations, FileOperations, Inode};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use heapless::String;
use rtl::error::ErrorType;

pub const ATTR_NORMAL_FILE: u8 = 0b0000000;
pub const ATTR_READ_ONLY: u8 = 0b00000001;
pub const ATTR_HIDDEN: u8 = 0b00000010;
pub const ATTR_SYSTEM: u8 = 0b00000100;
pub const ATTR_VOLUME_ID: u8 = 0b00001000;
pub const ATTR_DIRECTORY: u8 = 0b00010000;
pub const ATTR_ARCHIVE: u8 = 0b00100000;
pub const ATTR_LONG_NAME: u8 = 0b00001111;

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

#[async_trait::async_trait]
impl DirectoryOperations for Fat32Dir {
    async fn list(&self) -> Result<Vec<DirEntry>, ErrorType> {
        let mut cluster = alloc::vec![0; self.sb.cluster_size()];
        let mut res = vec![];

        self.sb
            .for_each_allocated_cluster_from(self.start, &mut cluster, |data| {
                let len = self.sb.cluster_size() / core::mem::size_of::<FsDirEntry>();
                let entries =
                    unsafe { core::slice::from_raw_parts(data.as_ptr() as *const FsDirEntry, len) };

                for entry in entries {
                    if entry.is_used() {
                        res.push(DirEntry {
                            name: String::from_utf8((&entry.name[1..]).try_into().unwrap())
                                .unwrap(),
                            flags: if entry.attr == ATTR_DIRECTORY {
                                DirEntryFlagsFlag::Directory
                            } else {
                                DirEntryFlagsFlag::File
                            }
                            .into(),
                        });
                    }
                }

                true
            })
            .await?;

        Ok(res)
    }
}
