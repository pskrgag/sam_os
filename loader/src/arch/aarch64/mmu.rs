use crate::mm::page_table::{PageKind, PagePerms};
use rtl::vmm::types::{Address, PhysAddr, VirtAddr};

pub const PTE_COUNT: usize = 512;
pub const PAGE_TABLE_LAST_LVL: usize = 3;
const TABLE_VALID: usize = 0b11;

// Kernel RW
const AP_UN_KRW: usize = 0b00;
// Kernel RO
const AP_UN_KRO: usize = 0b10;
// Privilege execute never
const PXN: usize = 1 << 53;
// User execute never
const UXN: usize = 1 << 54;
// Access bit
const ACCESS_BIT: usize = 1 << 10;

const PAGE_ENTRY_FLAGS_MASK: usize = 0xFFF0_0000_0000_0FFF;

#[repr(transparent)]
pub struct Pte(usize);

pub fn lvl_to_order(lvl: usize) -> usize {
    match lvl {
        0 => 39,
        1 => 30,
        2 => 21,
        3 => 12,
        _ => panic!("Only 4 levels are supported {:?}", lvl),
    }
}

pub fn va_to_index(va: VirtAddr, lvl: usize) -> usize {
    match lvl {
        0 => (usize::from(va) >> 39) & (PTE_COUNT - 1),
        1 => (usize::from(va) >> 30) & (PTE_COUNT - 1),
        2 => (usize::from(va) >> 21) & (PTE_COUNT - 1),
        3 => (usize::from(va) >> 12) & (PTE_COUNT - 1),
        _ => panic!("Wrong page table block index"),
    }
}

impl Pte {
    pub fn pa(&self) -> PhysAddr {
        PhysAddr::new(self.0 & !PAGE_ENTRY_FLAGS_MASK)
    }

    pub fn is_valid(&self) -> bool {
        self.0 & 0b11 != 0
    }

    pub fn new_non_leaf(next: PhysAddr) -> Self {
        Self(TABLE_VALID | next.bits())
    }

    pub fn make(pa: PhysAddr, perms: PagePerms, kind: PageKind) -> Self {
        const fn ap(perms: usize) -> usize {
            (perms << 6) as usize
        }
        const fn mair(idx: u8) -> usize {
            (idx << 2) as usize
        }

        let perms = match perms {
            PagePerms::Read => ap(AP_UN_KRO) | PXN,
            PagePerms::ReadWrite => ap(AP_UN_KRW) | PXN,
            PagePerms::Execute => ap(AP_UN_KRO),
        };

        let atts = match kind {
            PageKind::Normal => mair(0),
            PageKind::Device => mair(1),
        };

        Self(perms | ACCESS_BIT | atts | pa.bits() | 0b11)
    }

    pub fn bits(&self) -> usize {
        self.0
    }
}
