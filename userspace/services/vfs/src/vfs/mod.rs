use crate::bindings_BlkDev::BlkDev;
use crate::fs::inode::Inode;
use crate::fs::Filesystem;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use fs::path::Path;
use rtl::error::ErrorType;

pub struct Vfs {
    mountpoints: BTreeMap<String, Arc<dyn Inode>>,
}

impl Vfs {
    pub async fn new<S: AsRef<str>>(blk: BlkDev, fs: S) -> Result<Self, ErrorType> {
        let mut s = Self {
            mountpoints: BTreeMap::new(),
        };

        s.mount(blk, "/", fs).await?;
        Ok(s)
    }

    pub async fn mount<P: AsRef<Path>, S: AsRef<str>>(
        &mut self,
        blk: BlkDev,
        to: P,
        name: S,
    ) -> Result<(), ErrorType> {
        let sb = match name.as_ref() {
            "fat32" => crate::fs::fat32::Fat32::try_mount(blk).await?,
            _ => todo!(),
        };

        self.mountpoints.insert(to.as_ref().into_owned(), sb);
        Ok(())
    }
}
