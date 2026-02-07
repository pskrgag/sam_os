use super::dir::Fat32DirRef;
use super::sb::Cluster;
use crate::bindings_Vfs::{File, FileRequest};
use crate::vfs::inode::FileOperations;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use libc::vmm::{vm_object::VmObject, vms::vms};
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;
use rtl::vmm::MappingType;

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
        let mut file = self.inner.lock();

        file.extend_to_size(offset + buf.len()).await?;
        todo!()
    }

    async fn write(&self, buf: &[u8], offset: usize) -> Result<usize, ErrorType> {
        let mut buf = buf;
        let mut file = self.inner.lock();
        let sb = file.parent.super_block();
        let cluster_size = sb.cluster_size();

        file.extend_to_size(offset + buf.len()).await?;

        let first_cluster_idx = offset / cluster_size;
        let last_cluster_idx = (offset + buf.len()) / cluster_size;
        let mut cluster_offset = offset % cluster_size;

        for cl in first_cluster_idx..=last_cluster_idx {
            let mut cluster = alloc::vec![0; cluster_size];
            let to_copy = (cluster_size - cluster_offset).min(buf.len());
            let cl = Cluster(cl as _);

            sb.read_cluster(cl, &mut cluster).await?;
            cluster[cluster_offset..cluster_offset + to_copy].copy_from_slice(&buf[..to_copy]);

            buf = &buf[to_copy..];
            sb.write_cluster(cl, &cluster).await?;
            cluster_offset = 0;
        }

        assert!(buf.len() == 0);
        file.parent.update_size(buf.len()).await?;
        Ok(buf.len())
    }
}
