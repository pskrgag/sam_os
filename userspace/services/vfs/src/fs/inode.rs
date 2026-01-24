use crate::bindings_Vfs::DirEntry;
use alloc::sync::Arc;
use alloc::vec::Vec;
use rtl::error::ErrorType;
use alloc::boxed::Box;

#[async_trait::async_trait]
pub trait DirectoryOperations: Send + Sync {
    async fn list(&self) -> Result<Vec<DirEntry>, ErrorType>;
}

pub trait FileOperations: Send + Sync {
    fn read(&mut self, buf: &mut [u8], offset: u64) -> Result<usize, ErrorType>;
    fn write(&mut self, buf: &[u8], offset: u64) -> Result<usize, ErrorType>;
}

pub enum Inode {
    Directory(Arc<dyn DirectoryOperations>),
    File(Arc<dyn FileOperations>),
}
