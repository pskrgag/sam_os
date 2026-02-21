use super::dir::Fat32Dir;
use super::fat::FatEntry;
use super::fat_alloc::FatAlloc;
use crate::bindings_BlkDev::BlkDev;
use crate::vfs::inode::{Inode, InodeKind};
use crate::vfs::vfs;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::Add;
use core::ops::{BitOr, BitOrAssign};
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

#[repr(C)]
#[allow(dead_code)]
struct FatBootSector {
    ignored: [u8; 3],     /* Boot strap short or near jump */
    system_id: [u8; 8],   /* Name - can be used to special case partition manager volumes */
    sector_size: [u8; 2], /* bytes per logical sector */
    sec_per_clus: u8,     /* sectors/cluster */
    reserved: u16,        /* reserved sectors */
    fats: u8,             /* number of FATs */
    dir_entries: [u8; 2], /* root directory entries */
    sectors: [u8; 2],     /* number of sectors */
    media: u8,            /* media code */
    fat_length: u16,      /* sectors/FAT */
    secs_track: u16,      /* sectors per track */
    heads: u16,           /* number of heads */
    hidden: u32,          /* hidden sectors (unused) */
    total_sect: u32,      /* number of sectors (if sectors == 0) */
    length: u32,          /* sectors/FAT */
    flags: u16,           /* bit 8: fat mirroring, low 4: active fat */
    version: [u8; 2],     /* major, minor filesystem version */
    root_cluster: u32,    /* first cluster in root directory */
    info_sector: u16,     /* filesystem info sector */
    backup_boot: u16,     /* backup boot sector */
    reserved2: [u16; 6],  /* Unused */
    drive_number: u8,     /* Physical drive number */
    state: u8,            /* undocumented, but used for mount state. */
    signature: u8,        /* extended boot signature */
    vol_id: [u8; 4],      /* volume ID */
    vol_label: [u8; 11],  /* volume label */
    fs_type: [u8; 8],     /* file system type */
                          /* other fields are not added here */
}

#[allow(dead_code)]
struct FatBiosParamBlock {
    fat_sector_size: u16,
    fat_sec_per_clus: u8,
    fat_reserved: u16,
    fat_fats: u8,
    fat_dir_entries: u16,
    fat_sectors: u16,
    fat_fat_length: u16,
    fat_total_sect: u32,

    fat32_length: u32,
    fat32_root_cluster: u32,
    fat32_info_sector: u16,
    fat32_state: u8,
    fat32_vol_id: u32,
}

#[repr(C)]
struct BootFsInfo {
    signature1: u32,       /* 0x41615252L */
    reserved1: [u32; 120], /* Nothing as far as I can tell */
    signature2: u32,       /* 0x61417272L */
    free_clusters: u32,    /* Free cluster count.  -1 if unknown */
    next_cluster: u32,     /* Most recently allocated cluster */
    reserved2: [u32; 4],
}

pub fn data_from_bytes<T>(block: &[u8]) -> &T {
    assert!(block.len() >= size_of::<T>());

    // Fuck off we ball. Maybe unaligned but who cares?
    unsafe { &*(block.as_ptr() as usize as *const T) }
}

impl FatBiosParamBlock {
    fn from_block(block: &[u8]) -> Result<Self, ErrorType> {
        let boot_sector: &FatBootSector = data_from_bytes(block);

        let s = Self {
            fat_sector_size: u16::from_le_bytes(boot_sector.sector_size),
            fat_sec_per_clus: boot_sector.sec_per_clus,
            fat_reserved: boot_sector.reserved,
            fat_fats: boot_sector.fats,
            fat_dir_entries: u16::from_le_bytes(boot_sector.dir_entries),
            fat_sectors: u16::from_le_bytes(boot_sector.sectors),
            fat_fat_length: boot_sector.fat_length,
            fat_total_sect: boot_sector.total_sect,
            fat32_length: boot_sector.length,
            fat32_root_cluster: boot_sector.root_cluster,
            fat32_info_sector: boot_sector.info_sector,
            fat32_state: boot_sector.state,
            fat32_vol_id: u32::from_le_bytes(boot_sector.vol_id),
        };

        // Validate read data

        if s.fat_reserved == 0 {
            println!("Number of reserved sectors cannot be 0");
            return Err(ErrorType::InvalidArgument);
        }

        if s.fat_fats == 0 {
            println!("Number of FAT structure is 0");
            return Err(ErrorType::InvalidArgument);
        }

        if !(0xf8 <= boot_sector.media || boot_sector.media == 0xf0) {
            println!("Invalid media");
            return Err(ErrorType::InvalidArgument);
        }

        if s.fat_sector_size.next_power_of_two() != s.fat_sector_size
            || s.fat_sector_size < 512
            || s.fat_sector_size > 4096
        {
            println!("Invalid block size");
            return Err(ErrorType::InvalidArgument);
        }

        if s.fat_sec_per_clus.next_power_of_two() != s.fat_sec_per_clus {
            println!("Invalid sectors per cluster size");
            return Err(ErrorType::InvalidArgument);
        }

        if s.fat_fat_length == 0 && s.fat32_length == 0 {
            println!("Invalid fat length");
            return Err(ErrorType::InvalidArgument);
        }

        Ok(s)
    }
}

/// Sector
#[derive(Copy, Clone, Debug)]
pub struct Sector(u32);

impl Add<u32> for Sector {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

/// Cluster
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Cluster(pub(super) u32);

impl Into<Cluster> for u32 {
    fn into(self) -> Cluster {
        Cluster(self)
    }
}

/// Result of walk cb
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CallbackRes {
    StopSync = 3,
    ContinueSync = 2,
    Stop = 1,
    Continue = 0,
}

impl BitOr for CallbackRes {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let s = self as u8;
        let rhs = rhs as u8;

        unsafe { core::mem::transmute(s | rhs) }
    }
}

impl BitOrAssign for CallbackRes {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

/// Super-block reference
pub type SuperBlockRef = Arc<SuperBlock>;

/// Wrapper for block device
pub(super) struct BlockDevice(BlkDev);

impl BlockDevice {
    /// Reads one sector of data from the backing block device
    pub async fn read_sector(&self, sector: Sector, to: &mut [u8]) -> Result<(), ErrorType> {
        let res = self.0.ReadBlock(sector.0).await?;

        to.copy_from_slice(&res.data);
        Ok(())
    }

    /// Writes one sector of data to the backing block device
    pub async fn write_sector(&self, sector: Sector, from: &[u8]) -> Result<(), ErrorType> {
        self.0
            .WriteBlock(sector.0, from.try_into().unwrap())
            .await?;

        Ok(())
    }
}

/// Inner structure for FAT32 superblock.
/// Should be protected by the lock
pub(super) struct SuperBlockInner {
    /// Free clusters
    free_clusters: FatAlloc,
    /// The device
    blk: BlockDevice,
}

impl SuperBlockInner {
    pub async fn new(blk: BlkDev, fat_start: Sector, fat_length: usize) -> Result<Self, ErrorType> {
        let blk = BlockDevice(blk);
        let free_clusters = FatAlloc::new(fat_start, fat_length, &blk).await?;

        Ok(Self { free_clusters, blk })
    }

    /// Reads one sector of data from the backing block device
    pub async fn read_sector(&self, sector: Sector, to: &mut [u8]) -> Result<(), ErrorType> {
        self.blk.read_sector(sector, to).await
    }

    /// Writes one sector of data to the backing block device
    pub async fn write_sector(&self, sector: Sector, from: &[u8]) -> Result<(), ErrorType> {
        self.blk.write_sector(sector, from).await
    }

    pub async fn allocate_clusters(
        &mut self,
        start: Option<Cluster>,
        num_clusters: usize,
    ) -> Result<Vec<Cluster>, ErrorType> {
        self.free_clusters
            .allocate_clusters(start, num_clusters, &self.blk)
            .await
    }

    pub async fn lookup_cluster_chain(&self, start: Cluster) -> Result<Vec<Cluster>, ErrorType> {
        
    }
}

/// Fat32 superblock structure
pub struct SuperBlock {
    inner: Spinlock<SuperBlockInner>,
    /// Size of cluster
    cluster_size: usize,
    /// Size of FAT (file allocation table)
    fat_length: u32,
    /// Number of FATs
    fats: u8,
    /// Start of the first FAT
    fat_start: Sector,
    /// Cluster of the root directory
    root_cluster: Cluster,
    /// Data start sector
    data_start: Sector,
    /// Sectors per cluster
    sectors_per_cluster: u8,
}

impl SuperBlock {
    pub async fn from_dev(blk: BlkDev) -> Result<Self, ErrorType> {
        let sb = blk.ReadBlock(0).await?.data;
        let bpb = FatBiosParamBlock::from_block(&sb)?;

        let sectors_per_cluster = bpb.fat_sec_per_clus;
        let cluster_size = 512 * sectors_per_cluster as usize;
        let fats = bpb.fat_fats;
        let fat_start = bpb.fat_reserved;
        let mut fat_length = bpb.fat_fat_length as u32;
        let mut root_cluster = 0;
        // let mut free_clusters = None;

        if fat_length == 0 && bpb.fat32_length != 0 {
            fat_length = bpb.fat32_length;
            root_cluster = bpb.fat32_root_cluster;

            let fs_info = bpb.fat32_info_sector;
            let info = blk.ReadBlock(fs_info as _).await?;
            let info: &BootFsInfo = data_from_bytes(&info.data);

            if info.signature1 != 0x41615252 || info.signature2 != 0x61417272 {
                println!("Invalid FAT32 signature");
                return Err(ErrorType::InvalidArgument);
            }

            // free_clusters = (info.free_clusters != u32::MAX).then_some(info.free_clusters);
        }

        let data_start = fat_start + fats as u16 * fat_length as u16;

        println!("Cluster size {}", cluster_size);
        println!("Sectors per cluster {}", sectors_per_cluster);
        println!("Data start {}", data_start);
        println!("Root cluster {}", root_cluster);
        println!("Fat lenght {}", fat_length);

        Ok(Self {
            cluster_size,
            fat_length,
            fats,
            fat_start: Sector(fat_start as _),
            root_cluster: Cluster(root_cluster),
            data_start: Sector(data_start as u32),
            sectors_per_cluster,
            inner: Spinlock::new(
                SuperBlockInner::new(blk, Sector(fat_start as _), fat_length as _).await?,
            ),
        })
    }

    async fn fat_entry_for_cluster(
        self: &Arc<Self>,
        cluster: Cluster,
    ) -> Result<FatEntry, ErrorType> {
        let fats_per_sector = (self.sector_size() / core::mem::size_of::<FatEntry>()) as u32;
        let offset = cluster.0 % fats_per_sector;
        let sector = Sector(self.fat_start.0 + self.cluster_to_sector(cluster).0 / fats_per_sector);
        let mut sector_data = [0u8; 512];

        self.inner
            .lock()
            .read_sector(sector, &mut sector_data)
            .await?;

        let fats = unsafe {
            core::slice::from_raw_parts::<FatEntry>(
                sector_data.as_ptr() as _,
                fats_per_sector as usize,
            )
        };

        Ok(fats[offset as usize])
    }

    pub fn cluster_size(self: &Arc<Self>) -> usize {
        self.cluster_size
    }

    pub fn sector_size(self: &Arc<Self>) -> usize {
        512
    }

    pub async fn for_each_allocated_cluster_from<
        F: FnMut(&mut [u8]) -> CallbackRes + Send + Sync,
    >(
        self: &Arc<Self>,
        start: Cluster,
        to: &mut [u8],
        mut f: F,
    ) -> Result<(), ErrorType> {
        let mut start = Some(start);

        while let Some(s) = start {
            self.read_cluster(s, to).await?;

            let res = f(to);
            match res {
                CallbackRes::Stop => break,
                CallbackRes::Continue => {}
                CallbackRes::StopSync => {
                    self.write_cluster(s, to).await?;
                    break;
                }
                CallbackRes::ContinueSync => {
                    self.write_cluster(s, to).await?;
                }
            }

            let entry = self.fat_entry_for_cluster(s).await?;
            start = entry.next()
        }

        Ok(())
    }

    fn cluster_to_sector(&self, cluster: Cluster) -> Sector {
        Sector((cluster.0 - 2) * (self.sectors_per_cluster as u32) + self.data_start.0)
    }

    pub async fn read_cluster(
        self: &Arc<Self>,
        cluster: Cluster,
        to: &mut [u8],
    ) -> Result<(), ErrorType> {
        assert!(cluster.0 >= 2);
        // TODO: fuck off I am too lazy
        assert!(self.cluster_size == 512);

        self.inner
            .lock()
            .read_sector(self.cluster_to_sector(cluster), to)
            .await
    }

    pub async fn write_cluster(
        self: &Arc<Self>,
        cluster: Cluster,
        from: &[u8],
    ) -> Result<(), ErrorType> {
        assert!(cluster.0 >= 2);
        // TODO: fuck off I am too lazy
        assert!(self.cluster_size == 512);

        self.inner
            .lock()
            .write_sector(self.cluster_to_sector(cluster), from)
            .await?;
        Ok(())
    }

    pub async fn root(
        self: &Arc<Self>,
        parent: Option<Arc<Inode>>,
    ) -> Result<Arc<Inode>, ErrorType> {
        Fat32Dir::new(self.clone(), self.root_cluster)
            .await
            .map(|x| Inode::new(InodeKind::Directory(Arc::new(x)), parent))
    }

    /// Allocates clusters and links them to chain starting from start
    pub(super) async fn allocate_clusters(
        &self,
        start: Option<Cluster>,
        num_clusters: usize,
    ) -> Result<Vec<Cluster>, ErrorType> {
        self.inner
            .lock()
            .allocate_clusters(start, num_clusters)
            .await
    }
}
