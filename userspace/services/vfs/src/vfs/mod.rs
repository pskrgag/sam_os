use crate::bindings_BlkDev::BlkDev;
use crate::fs::inode::Inode;
use crate::fs::Filesystem;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use fs::path::Path;
use libc::handle::Handle;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

mod dir;

pub struct Vfs {
    mountpoints: Spinlock<BTreeMap<String, Arc<Inode>>>,
}

impl Vfs {
    /// Creates new VFS with specified root device
    pub async fn new<S: AsRef<str>>(blk: BlkDev, fs: S) -> Result<Self, ErrorType> {
        let mut s = Self {
            mountpoints: Spinlock::new(BTreeMap::new()),
        };

        s.mount(blk, "/", fs).await?;
        Ok(s)
    }

    pub fn open_dir<S: AsRef<str>>(
        &self,
        dir: S,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        if let Some(mp) = self.mountpoints.lock().get(dir.as_ref()) {
            dir::OpenDirectory::new(mp.clone())
        } else {
            todo!()
        }
    }

    /// Mount device on specified mount point
    pub async fn mount<P: AsRef<Path>, S: AsRef<str>>(
        &mut self,
        blk: BlkDev,
        to: P,
        name: S,
    ) -> Result<(), ErrorType> {
        let sb = match name.as_ref() {
            "fat32" => crate::fs::fat32::Fat32::try_mount(blk).await?,
            _ => panic!("Unknown FS"),
        };

        self.mountpoints.lock().insert(to.as_ref().into_owned(), sb);
        Ok(())
    }
}
