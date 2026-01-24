use alloc::sync::Arc;
use rtl::error::ErrorType;

pub trait DirectoryOperations: Send + Sync {
    fn open(&self, path: &str) -> Result<Inode, ErrorType>;
}

pub trait FileOperations: Send + Sync {
    fn read(&mut self, buf: &mut [u8], offset: u64) -> Result<usize, ErrorType>;
    fn write(&mut self, buf: &[u8], offset: u64) -> Result<usize, ErrorType>;
}

pub enum Inode {
    Directory(Arc<dyn DirectoryOperations>),
    File(Arc<dyn FileOperations>),
}
