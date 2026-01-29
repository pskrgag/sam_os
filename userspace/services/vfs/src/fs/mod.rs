use crate::bindings_BlkDev::BlkDev;
use crate::vfs::inode::Inode;
use alloc::sync::Arc;
use rtl::error::ErrorType;

pub mod fat32;

pub trait Filesystem {
    /// Mounts device and return root directory inode
    async fn try_mount(blk: BlkDev, parent: Option<Arc<Inode>>) -> Result<Arc<Inode>, ErrorType>;
}
