use crate::bindings_BlkDev::BlkDev;
use crate::fs::Filesystem;
use crate::vfs::inode::DirectoryOperations;
use crate::vfs::inode::FileOperations;
use alloc::sync::Arc;
use dcache::Dentry;
use dcache::CreateType;
use libc::handle::Handle;
use rtl::error::ErrorType;
use spin::once::Once;

mod dcache;
mod dir;
mod file;
pub mod inode;

pub struct Vfs {
    root: Arc<Dentry>,
}

static VFS: Once<Arc<Vfs>> = Once::new();

pub type Directory = Arc<dyn DirectoryOperations>;
pub type File = Arc<dyn FileOperations>;

impl Vfs {
    /// Creates new VFS with specified root device
    pub async fn new<S: AsRef<str>>(blk: BlkDev, fs: S) -> Result<Self, ErrorType> {
        let sb = match fs.as_ref() {
            "fat32" => crate::fs::fat32::Fat32::try_mount(blk).await?,
            _ => panic!("Unknown FS"),
        };
        let s = Self {
            root: Dentry::new_root(sb),
        };

        Ok(s)
    }

    /// Opens a directory
    pub async fn root(
        &self,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>> + use<>, Handle), ErrorType> {
        dir::OpenDirectory::new(self.root.clone())
    }
}

pub fn vfs() -> &'static Arc<Vfs> {
    unsafe { VFS.get_unchecked() }
}

pub async fn init<S: AsRef<str>>(blk: BlkDev, fs: S) {
    let vfs = Arc::new(Vfs::new(blk, fs).await.unwrap());

    VFS.call_once(|| vfs);
}
