use super::sb::Cluster;

const FAT_FREE: u32 = 0x00000000; // Available for use
const FAT_RESERVED: u32 = 0x00000001; // Reserved cluster
const FAT_BAD: u32 = 0x0FFFFFF7; // Bad cluster (don't use)
const FAT_EOF_MIN: u32 = 0x0FFFFFF8; // End of chain markers
const FAT_EOF_MAX: u32 = 0x0FFFFFFF;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct FatEntry(u32);

impl FatEntry {
    pub fn is_free(&self) -> bool {
        self.0 == FAT_FREE
    }

    pub fn next(&self) -> Option<Cluster> {
        let val = self.0;

        if val != FAT_FREE
            && val != FAT_RESERVED
            && val != FAT_BAD
            && val != FAT_EOF_MIN
            && val != FAT_EOF_MAX
        {
            Some(Cluster(val & ((1 << 28) - 1)))
        } else {
            None
        }
    }
}

// vfs :: ffffff8   0
// vfs :: fffffff   1
// vfs :: ffffff8   2
// vfs :: 0
