use downcast_rs::{impl_downcast, Downcast};
use rtl::error::ErrorType;

impl_downcast!(Inode);

pub trait DirectoryOperations: Inode + Send + Sync {}

pub trait FileOperations: Inode + Send + Sync {
    fn read(&mut self, buf: &mut [u8], offset: u64) -> Result<usize, ErrorType>;
    fn write(&mut self, buf: &[u8], offset: u64) -> Result<usize, ErrorType>;
}

pub trait Inode: Send + Sync + Downcast {}
