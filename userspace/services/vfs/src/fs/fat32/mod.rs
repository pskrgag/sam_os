use super::Filesystem;
use crate::vfs::inode::Inode;
use crate::BlkDev;
use alloc::sync::Arc;
use rtl::error::ErrorType;
use sb::SuperBlock;

mod dir;
mod fat;
mod fat_alloc;
mod file;
mod inode;
mod sb;

pub struct Fat32;

impl Filesystem for Fat32 {
    async fn try_mount(blk: BlkDev, parent: Option<Arc<Inode>>) -> Result<Arc<Inode>, ErrorType> {
        blk.SetBlockSize(512).await?;

        let sb = Arc::new(SuperBlock::from_dev(blk).await?);
        let root = sb.root(parent).await?;

        Ok(root)
    }
}
