use super::file::OpenFile;
use crate::bindings_Vfs::DirEntry;
use adt::GrowBitAllocator;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::pin::Pin;
use fs::path::Path;
use libc::handle::Handle;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

#[async_trait::async_trait]
pub trait DirectoryOperations: Send + Sync {
    /// List all entries in a given directory
    async fn list(&self) -> Result<Vec<DirEntry>, ErrorType>;

    /// Lookup the entry
    async fn lookup(&self, path: &Path) -> Result<Inode, ErrorType>;

    /// Creates a new file in the directory. Returns a handle to file
    async fn create_file(&self, name: &str, parent: &Arc<Inode>) -> Result<Arc<Inode>, ErrorType>;
}

#[async_trait::async_trait]
pub trait FileOperations: Send + Sync {
    /// Read data from the file
    async fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, ErrorType>;

    /// Write data to the file
    async fn write(&self, buf: &[u8], offset: usize) -> Result<usize, ErrorType>;
}

pub enum InodeKind {
    Directory(Arc<dyn DirectoryOperations>),
    File(Arc<dyn FileOperations>),
}

pub struct Inode {
    num: usize,
    kind: InodeKind,
    parent: Option<Arc<Inode>>,
}

impl Inode {
    pub fn new(kind: InodeKind, parent: Option<Arc<Inode>>) -> Arc<Self> {
        static ID_ALLOCATOR: Spinlock<GrowBitAllocator> = Spinlock::new(GrowBitAllocator::empty());

        Arc::new(Self {
            num: ID_ALLOCATOR.lock().allocate(),
            kind,
            parent,
        })
    }

    pub fn as_dir(&self) -> Option<&Arc<dyn DirectoryOperations>> {
        match &self.kind {
            InodeKind::Directory(d) => Some(d),
            _ => None,
        }
    }

    pub fn is_dir(&self) -> bool {
        self.as_dir().is_some()
    }

    pub fn is_file(&self) -> bool {
        !self.is_dir()
    }

    pub fn kind(&self) -> &InodeKind {
        &self.kind
    }
}
