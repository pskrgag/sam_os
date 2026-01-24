use super::dir::{Fat32Dir, FsDirEntry};
use crate::bindings_BlkDev::BlkDev;
use crate::fs::inode::Inode;
use alloc::sync::Arc;
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

/// Cluster
#[derive(Copy, Clone, Debug)]
pub struct Cluster(u32);

impl Into<Cluster> for u32 {
    fn into(self) -> Cluster {
        Cluster(self)
    }
}

/// Super-block reference
pub type SuperBlockRef = Arc<SuperBlock>;

/// Fat32 superblock structure
struct SuperBlockInner {
    /// Free clusters
    free_clusters: Option<u32>,
    /// The device
    blk: BlkDev,
}

pub struct SuperBlock {
    inner: Spinlock<SuperBlockInner>,
    /// Size of cluster
    cluster_size: usize,
    /// Size of FAT (file allocation table)
    fat_length: u32,
    /// Number of FATs
    fats: u8,
    /// Start of the first FAT
    fat_start: u16,
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
        let mut free_clusters = None;

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

            free_clusters = (info.free_clusters != u32::MAX).then_some(info.free_clusters);
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
            fat_start,
            root_cluster: Cluster(root_cluster),
            data_start: Sector(data_start as u32),
            sectors_per_cluster,
            inner: Spinlock::new(SuperBlockInner { free_clusters, blk }),
        })
    }

    pub fn cluster_size(self: &Arc<Self>) -> usize {
        self.cluster_size
    }

    pub async fn read_cluster(
        self: &Arc<Self>,
        cluster: Cluster,
        to: &mut [u8],
    ) -> Result<(), ErrorType> {
        assert!(cluster.0 >= 2);
        // TODO: fuck off I am too lazy
        assert!(self.cluster_size == 512);

        let sector = (cluster.0 - 2) * (self.sectors_per_cluster as u32) + self.data_start.0;
        let res = self.inner.lock().blk.ReadBlock(sector).await?;

        to.copy_from_slice(&res.data);
        Ok(())
    }

    pub async fn root(self: &Arc<Self>) -> Result<Arc<Inode>, ErrorType> {
        Ok(Arc::new(Inode::Directory(Arc::new(
            Fat32Dir::new(self.clone(), self.root_cluster).await?,
        ))))
    }
}
