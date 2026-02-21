use super::dir::Fat32DirRef;
use super::sb::Cluster;
use crate::vfs::inode::FileOperations;
use alloc::boxed::Box;
use alloc::vec::Vec;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub struct FatFileInner {
    allocated_clusters: Vec<Cluster>,
    parent: Fat32DirRef,
}

pub struct FatFile {
    inner: Spinlock<FatFileInner>,
}

impl FatFileInner {
    async fn extend_to_size(&mut self, size: usize) -> Result<(), ErrorType> {
        let sb = self.parent.super_block();
        let cluster_size = sb.cluster_size();
        let last_cluster_idx = size.next_multiple_of(cluster_size) / cluster_size;

        if self.allocated_clusters.len() < last_cluster_idx {
            let to_allocated = last_cluster_idx - self.allocated_clusters.len();
            let chain = self.allocated_clusters.last().cloned();

            self.allocated_clusters
                .extend(self.parent.allocate_clusters(chain, to_allocated).await?);
        }

        Ok(())
    }

    async fn for_file_range<F: FnMut(&mut [u8]) -> bool>(
        &self,
        start: usize,
        size: usize,
        mut f: F,
    ) -> Result<usize, ErrorType> {
        let sb = self.parent.super_block();
        let cluster_size = sb.cluster_size();
        let first_cluster_idx = start / cluster_size;
        let last_cluster_idx = (start + size) / cluster_size;
        let mut cluster_offset = start % cluster_size;
        let mut processed = 0;
        let mut size = size;

        for cl in first_cluster_idx..=last_cluster_idx.min(self.allocated_clusters.len() - 1) {
            let mut cluster = alloc::vec![0; cluster_size];
            let cluster_range_size = (cluster_size - cluster_offset).min(size);
            let cl = self.allocated_clusters[cl];

            sb.read_cluster(cl, &mut cluster).await?;

            if f(&mut cluster[cluster_offset..cluster_offset + cluster_range_size]) {
                sb.write_cluster(cl, &cluster).await?;
            }

            cluster_offset = 0;
            processed += cluster_range_size;
            size -= cluster_range_size;
        }

        Ok(processed)
    }
}

impl FatFile {
    pub fn new(allocated_clusters: Vec<Cluster>, parent: Fat32DirRef) -> Self {
        Self {
            inner: Spinlock::new(FatFileInner {
                allocated_clusters,
                parent,
            }),
        }
    }
}

#[async_trait::async_trait]
impl FileOperations for FatFile {
    async fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, ErrorType> {
        let file = self.inner.lock();
        let mut processed = 0;
        let max_read = file.parent.size() as usize - offset;

        let read = file
            .for_file_range(offset, buf.len(), |cluster| {
                let buf_len = cluster.len();

                buf[processed..processed + buf_len].copy_from_slice(cluster);
                processed += buf_len;
                false
            })
            .await?;

        Ok(read.min(max_read))
    }

    async fn write(&self, buf: &[u8], offset: usize) -> Result<usize, ErrorType> {
        let mut buf = buf;
        let mut file = self.inner.lock();

        file.extend_to_size(offset + buf.len()).await?;

        let processed = file
            .for_file_range(offset, buf.len(), |cluster| {
                let buf_len = cluster.len();

                cluster.copy_from_slice(&buf[..buf_len]);
                buf = &buf[buf_len..];
                true
            })
            .await?;

        assert!(buf.len() == 0);
        file.parent.update_size(processed as isize).await?;
        Ok(processed)
    }
}
