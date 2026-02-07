//! FAT entries allocator

use super::fat::FatEntry;
use super::sb::{BlockDevice, Cluster, Sector};
use adt::BitAllocator;
use alloc::vec::Vec;
use rtl::error::ErrorType;

pub(super) struct FatAlloc {
    /// Start of the FAT table
    fat_start: Sector,
    /// Fat lenght (in sectors)
    length: usize,
    /// Cache of FAT
    cache: BitAllocator,
}

impl FatAlloc {
    pub async fn new(
        fat_start: Sector,
        length: usize,
        blk: &BlockDevice,
    ) -> Result<Self, ErrorType> {
        Self::setup_cache(fat_start, length, blk)
            .await
            .map(|cache| Self {
                fat_start,
                length,
                cache,
            })
    }

    fn fats_per_sector() -> u32 {
        (512 / core::mem::size_of::<FatEntry>()) as _
    }

    /// Reads data from the block device and caches it.
    pub async fn setup_cache(
        fat_start: Sector,
        fat_length: usize,
        blk: &BlockDevice,
    ) -> Result<BitAllocator, ErrorType> {
        let mut new_cache = BitAllocator::new(512 / core::mem::size_of::<FatEntry>() * fat_length);

        for i in 0..fat_length {
            let mut sector = alloc::vec![0; 512];

            blk.read_sector(fat_start + i as _, &mut sector).await?;
            let fats = unsafe {
                core::slice::from_raw_parts::<FatEntry>(
                    sector.as_ptr() as _,
                    Self::fats_per_sector() as usize,
                )
            };

            for (j, fat) in fats.iter().enumerate() {
                if !fat.is_free() {
                    println!("Allocated fat {}", Self::fats_per_sector() as usize * i + j);
                    new_cache
                        .allocate_specific(Self::fats_per_sector() as usize * i + j)
                        .unwrap();
                }
            }
        }

        Ok(new_cache)
    }

    async fn commit_chain(&mut self, cl: &[Cluster], blk: &BlockDevice) -> Result<(), ErrorType> {
        let mut sector = alloc::vec![0; 512];
        let fat_sector = self.fat_start + cl[0].0 / Self::fats_per_sector();
        let fat_offset = (cl[0].0 % Self::fats_per_sector()) as usize;

        blk.read_sector(fat_sector, &mut sector).await?;

        let fats = unsafe {
            core::slice::from_raw_parts_mut::<FatEntry>(
                sector.as_mut_ptr() as _,
                Self::fats_per_sector() as usize,
            )
        };

        assert!(fats[fat_offset].is_free());

        if cl.len() == 2 {
            fats[fat_offset] = FatEntry::new_chain(cl[1]);
        } else {
            fats[fat_offset] = FatEntry::new_tail();
        }

        blk.write_sector(fat_sector, &sector).await?;
        Ok(())
    }

    async fn commit_cluster_chain(
        &mut self,
        start: Option<Cluster>,
        clusters: &[Cluster],
        blk: &BlockDevice,
    ) -> Result<(), ErrorType> {
        if let Some(start) = start {
            let chain = [start, clusters[0]];
            self.commit_chain(&chain, blk).await?;
        }

        for cl in clusters.windows(2) {
            self.commit_chain(cl, blk).await?;
        }

        Ok(())
    }

    pub async fn allocate_clusters(
        &mut self,
        start: Option<Cluster>,
        num_clusters: usize,
        blk: &BlockDevice,
    ) -> Result<Vec<Cluster>, ErrorType> {
        let mut res = alloc::vec![Cluster::default(); num_clusters];

        for i in 0..num_clusters {
            res[i] = Cluster(self.cache.allocate().ok_or(ErrorType::NoMemory)? as _);
        }

        match self.commit_cluster_chain(start, &res, blk).await {
            Ok(_) => {}
            Err(e) => {
                for cl in res {
                    self.cache.free(cl.0 as _);
                }

                return Err(e);
            }
        }

        Ok(res)
    }
}
