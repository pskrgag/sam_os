use crate::bindings_BlkDev::BlkDev;
use rtl::error::ErrorType;
use crate::fs::inode::Inode;
use alloc::sync::Arc;

pub mod fat32;
pub mod inode;

pub trait Filesystem {
    /// Mounts device and return root directory inode
    async fn try_mount(blk: BlkDev) -> Result<Arc<dyn Inode>, ErrorType>;
}
