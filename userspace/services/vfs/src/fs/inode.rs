use crate::bindings_Vfs::DirEntry;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use libc::handle::Handle;
use rtl::error::ErrorType;
use core::pin::Pin;

pub struct OpenFile {
    pub handler: Pin<Box<dyn Future<Output = Result<(), ErrorType>> + Send>>,
    pub handle: Handle,
}

#[async_trait::async_trait]
pub trait DirectoryOperations: Send + Sync {
    /// List all entries in a given directory
    async fn list(&self) -> Result<Vec<DirEntry>, ErrorType>;

    /// Creates a new file in the directory. Returns a handle to file
    async fn create_file(&self, name: &str) -> Result<OpenFile, ErrorType>;
}

pub trait FileOperations: Send + Sync {
    fn read(&mut self, buf: &mut [u8], offset: u64) -> Result<usize, ErrorType>;
    fn write(&mut self, buf: &[u8], offset: u64) -> Result<usize, ErrorType>;
}

pub enum Inode {
    Directory(Arc<dyn DirectoryOperations>),
    File(Arc<dyn FileOperations>),
}
