use crate::mm::{
    alloc::alloc_pages,
    page_table::{PageKind, PagePerms, PageTable},
};
use elf::{
    ElfBytes,
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
};
use hal::address::{Address, MemRange, PhysAddr, VirtAddr};
use hal::arch::PAGE_SIZE;

#[repr(align(0x1000))]
struct Aligned;

static KERNEL_BIN: &[u8] = rtl::include_bytes_align_as!(Aligned, env!("KERNEL_PATH"));
static mut KERNEL_EP: Option<usize> = None;

// Maps kernel and returns address of page table base
pub fn map_kernel(tt: &mut PageTable) {
    let elf =
        ElfBytes::<LittleEndian>::minimal_parse(KERNEL_BIN).expect("Failed to parse kernel elf");
    let phys_base = KERNEL_BIN.as_ptr();

    unsafe { KERNEL_EP = Some(elf.ehdr.e_entry as usize) };

    // TODO: Check that KERNEL_EP is indeed lies in Image range

    for seg in elf
        .segments()
        .unwrap()
        .into_iter()
        .filter(|phdr| phdr.p_type == PT_LOAD)
    {
        let base = seg.p_vaddr;
        let size = seg.p_memsz;
        let mut virt_range = MemRange::new(VirtAddr::from_bits(base as usize), size as usize);

        virt_range.align_page();

        let mut phys_range = if seg.p_filesz != 0 {
            MemRange::new(
                PhysAddr::from_bits(unsafe { phys_base.add(seg.p_offset as usize) } as usize),
                seg.p_filesz as usize,
            )
        } else {
            MemRange::new(
                alloc_pages(seg.p_memsz as usize / PAGE_SIZE).unwrap(),
                seg.p_memsz as usize,
            )
        };

        phys_range.align_page();

        let perms = if seg.p_flags == PF_W | PF_R {
            PagePerms::ReadWrite
        } else if seg.p_flags == PF_X | PF_R {
            PagePerms::Execute
        } else if seg.p_flags == PF_R {
            PagePerms::Read
        } else {
            panic!("Unknown elf permissions");
        };

        tt.map_pages(virt_range, phys_range, perms, PageKind::Normal);
    }

    info!("Mapped kernel image\n");
}

pub fn kernel_ep() -> VirtAddr {
    unsafe { VirtAddr::from_bits(KERNEL_EP.unwrap()) }
}
