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
        let last_cluster_idx = size / cluster_size;

        if self.allocated_clusters.len() < last_cluster_idx {
            let to_allocated = last_cluster_idx - self.allocated_clusters.len() + 1;
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
        let mut file = self.inner.lock();
        let sb = file.parent.super_block();
        let cluster_size = sb.cluster_size();

        file.extend_to_size(offset + buf.len()).await?;

        let first_cluster = offset / cluster_size;
        let first_cluster_offset = offset % cluster_size;
        let last_cluster = (offset + buf.len()) / cluster_size;
        let mut cluster = alloc::vec![0; cluster_size];

        if first_cluster == last_cluster {
            let first_cluster = Cluster(first_cluster as _);

            sb.read_cluster(first_cluster, &mut cluster).await?;
            cluster[first_cluster_offset..].copy_from_slice(buf);
            sb.write_cluster(first_cluster, &cluster).await?;
        } else {
            todo!()
            // for cl in first_cluster..=last_cluster {
            //     let cluster = Cluster(cl as _);
            //
            //     if cl == first_cluster && first_cluster_offset != 0 {
            //         sb.read_cluster(first_cluster, cluster).await?;
            //         cluster[first_cluster_offset..].copy_from_slice(buf);
            //         sb.write_cluster(first_cluster, cluster).await?;
            //     }
            //
            //     sb.write_cluster(Cluster(cl as u32), &mut cluster).await?;
            // }
        }

        Ok(buf.len())
    }
}
