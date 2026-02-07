use super::file::FatFile;
use super::sb::{CallbackRes, Cluster, SuperBlockRef};
use crate::bindings_Vfs::{DirEntry, DirEntryFlagsFlag};
use crate::vfs::inode::{DirectoryOperations, Inode, InodeKind};
use crate::vfs::vfs;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ffi::c_str::CStr;
use fs::path::Path;
use heapless::String;
use rtl::error::ErrorType;

pub const ATTR_NORMAL_FILE: u8 = 0b0000000;
// pub const ATTR_READ_ONLY: u8 = 0b00000001;
// pub const ATTR_HIDDEN: u8 = 0b00000010;
// pub const ATTR_SYSTEM: u8 = 0b00000100;
// pub const ATTR_VOLUME_ID: u8 = 0b00001000;
pub const ATTR_DIRECTORY: u8 = 0b00010000;
// pub const ATTR_ARCHIVE: u8 = 0b00100000;
// pub const ATTR_LONG_NAME: u8 = 0b00001111;

// On disk representation
#[repr(C)]
#[derive(Debug, Default)]
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
    pub fn is_free(&self) -> bool {
        self.name[0] == 0 || self.name[0] == 0xe5
    }

    pub fn first_cluster(&self) -> Cluster {
        Cluster(self.start as u32 | (self.starthi as u32) << 16)
    }

    pub fn is_dir(&self) -> bool {
        (self.attr & ATTR_DIRECTORY) != 0
    }

    pub fn new_empty_file(name: &str) -> Self {
        let mut real_name: [u8; 11] = [0; 11];

        real_name[..name.len()].copy_from_slice(name.as_bytes());

        Self {
            name: real_name,
            attr: ATTR_NORMAL_FILE,
            ..Default::default()
        }
    }
}

struct Fat32DirInner {
    sb: SuperBlockRef,
    start: Cluster,
}

#[derive(Clone)]
pub struct Fat32Dir {
    inner: Arc<Fat32DirInner>,
}

pub(super) struct Fat32DirRef {
    dir: Fat32Dir,
    size: u32,
    offset: usize,
}

impl Fat32DirRef {
    pub async fn allocate_clusters(
        &mut self,
        start: Option<Cluster>,
        num: usize,
    ) -> Result<Vec<Cluster>, ErrorType> {
        let res = self.dir.super_block().allocate_clusters(start, num).await?;

        self.dir
            .update_entry(self.offset, |entry| {
                if entry.start == 0 {
                    assert!(entry.starthi == 0);

                    entry.start = (res[0].0 & 0xFFFF) as u16;
                    entry.starthi = ((res[0].0 >> 16) & 0xFFFF) as u16;
                }
            })
            .await?;

        Ok(res)
    }

    pub async fn update_size(&mut self, size: usize) -> Result<(), ErrorType> {
        self.dir
            .update_entry(self.offset, |entry| {
                entry.size += size as u32;
            })
            .await?;
        Ok(())
    }

    pub fn super_block(&self) -> SuperBlockRef {
        self.dir.super_block()
    }
}

impl Fat32Dir {
    pub async fn new(sb: SuperBlockRef, start: Cluster) -> Result<Self, ErrorType> {
        Ok(Self {
            inner: Arc::new(Fat32DirInner { sb, start }),
        })
    }

    pub fn super_block(&self) -> SuperBlockRef {
        self.inner.sb.clone()
    }

    // TODO: directory should cache allocated clusters. Doing a linked list walk is a shitty
    // solution. But it's just fine for testing
    async fn update_entry<F: FnMut(&mut FsDirEntry) + Send + Sync>(
        &self,
        idx: usize,
        mut f: F,
    ) -> Result<(), ErrorType> {
        let mut cluster = alloc::vec![0; self.super_block().cluster_size()];

        let entries_per_cluster =
            self.super_block().cluster_size() / core::mem::size_of::<FsDirEntry>();
        let cl_idx = idx / entries_per_cluster;
        let cl_offset = idx % entries_per_cluster;
        let mut idx = 0;

        self.inner
            .sb
            .for_each_allocated_cluster_from(self.inner.start, &mut cluster, |data| {
                let mut res = CallbackRes::Continue;

                if idx == cl_idx {
                    let entries = unsafe {
                        core::slice::from_raw_parts_mut(
                            data.as_mut_ptr() as *mut FsDirEntry,
                            entries_per_cluster,
                        )
                    };

                    f(&mut entries[cl_offset]);
                    res = CallbackRes::StopSync;
                }

                idx += 1;
                res
            })
            .await?;

        Ok(())
    }

    async fn lookup_dir(&self, name: &str) -> Result<Arc<Inode>, ErrorType> {
        let mut res = Err(ErrorType::NotFound);

        self.for_each_dir_entry(|entry, _| {
            // if entry.is_dir() {
            //     let cstr = unsafe { CStr::from_ptr(entry.name.as_ptr()) }
            //         .to_str()
            //         .expect("Invalid entry name on FS");
            //     let dir = Self::new(self.super_block(), entry.first_cluster()).await?;
            //
            //     if cstr == first {
            //         res = Ok(Inode::new(
            //             InodeKind::Directory(Arc::new(dir)),
            //             Some(parent.clone()),
            //         ));
            //
            //         return CallbackRes::Stop;
            //     }
            // }

            CallbackRes::Continue
        })
        .await?;

        res
    }

    async fn for_each_dir_entry<F: FnMut(&mut FsDirEntry, usize) -> CallbackRes + Send + Sync>(
        &self,
        mut f: F,
    ) -> Result<(), ErrorType> {
        let mut cluster = alloc::vec![0; self.inner.sb.cluster_size()];
        let mut idx = 0;

        self.inner
            .sb
            .for_each_allocated_cluster_from(self.inner.start, &mut cluster, |data| {
                let len = self.inner.sb.cluster_size() / core::mem::size_of::<FsDirEntry>();
                let entries = unsafe {
                    core::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut FsDirEntry, len)
                };
                let mut res = CallbackRes::Continue;

                for entry in entries {
                    res |= f(entry, idx);

                    if matches!(res, CallbackRes::Stop | CallbackRes::StopSync) {
                        break;
                    }

                    idx += 1;
                }

                res
            })
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl DirectoryOperations for Fat32Dir {
    async fn list(&self) -> Result<Vec<DirEntry>, ErrorType> {
        let mut res = alloc::vec![];

        self.for_each_dir_entry(|entry, _| {
            if !entry.is_free() {
                res.push(DirEntry {
                    name: String::from_utf8((&entry.name[1..]).try_into().unwrap()).unwrap(),
                    flags: if entry.attr == ATTR_DIRECTORY {
                        DirEntryFlagsFlag::Directory
                    } else {
                        DirEntryFlagsFlag::File
                    }
                    .into(),
                });
            }

            CallbackRes::Continue
        })
        .await?;

        Ok(res)
    }

    async fn lookup(&self, path: &Path, parent: &Arc<Inode>) -> Result<Inode, ErrorType> {
        let components = path.components().collect::<Vec<_>>();

        self.for_each_dir_entry(|entry, _| todo!()).await?;

        todo!()
    }

    async fn create_file(&self, name: &str, parent: &Arc<Inode>) -> Result<Arc<Inode>, ErrorType> {
        let mut allocated_idx = 0;

        self.for_each_dir_entry(|entry, idx| {
            if entry.is_free() {
                *entry = FsDirEntry::new_empty_file(name);
                allocated_idx = idx;
                return CallbackRes::StopSync;
            }

            CallbackRes::Continue
        })
        .await?;

        let file = FatFile::new(
            alloc::vec::Vec::new(),
            Fat32DirRef {
                dir: self.clone(),
                offset: allocated_idx,
                size: 0,
            },
        );

        Ok(Inode::new(
            InodeKind::File(Arc::new(file)),
            Some(parent.clone()),
        ))
    }
}
