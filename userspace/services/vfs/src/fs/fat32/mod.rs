use super::Filesystem;
use crate::BlkDev;
use alloc::sync::Arc;
use rtl::error::ErrorType;
use sb::SuperBlock;
use crate::fs::inode::Inode;

mod dir;
mod inode;
mod sb;
mod fat;
mod file;

pub struct Fat32;

impl Filesystem for Fat32 {
    async fn try_mount(blk: BlkDev) -> Result<Arc<Inode>, ErrorType> {
        blk.SetBlockSize(512).await?;

        let sb = Arc::new(SuperBlock::from_dev(blk).await?);
        let root = sb.root().await?;

        Ok(root)
    }
}
