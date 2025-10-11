use crate::mm::allocators::page_alloc::page_allocator;
use alloc::vec::Vec;
use elf::{
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
    ElfBytes,
};
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

#[derive(Debug)]
pub struct Segment {
    pub va: MemRange<VirtAddr>,
    pub pa: MemRange<PhysAddr>,
    pub tp: MappingType,
}

#[derive(Debug)]
pub struct ElfData {
    pub regions: Vec<Segment>,
    pub ep: VirtAddr,
}

pub fn parse_initial_task(prot: &loader_protocol::LoaderArg) -> Option<ElfData> {
    let elf_data = unsafe {
        core::slice::from_raw_parts(
            prot.init_virt_task_base.0 as *const u8,
            prot.init_virt_task_base.1,
        )
    };
    let elf =
        ElfBytes::<LittleEndian>::minimal_parse(elf_data).expect("Failed to parse kernel elf");
    let mut secs = Vec::new();

    for seg in elf
        .segments()
        .unwrap()
        .into_iter()
        .filter(|phdr| phdr.p_type == PT_LOAD)
    {
        let base = seg.p_vaddr;
        let size = seg.p_memsz;
        let mut virt_range = MemRange::new(VirtAddr::new(base as usize), size as usize);

        virt_range.align_page();

        let phys_range = {
            let new_pages = MemRange::new(
                page_allocator()
                    .alloc(virt_range.size() as usize / PAGE_SIZE)
                    .unwrap(),
                virt_range.size(),
            );

            if seg.p_memsz != 0 {
                let mut start = VirtAddr::from(new_pages.start());
                let start = unsafe { start.as_slice_mut::<u8>(virt_range.size()) };
                let elf_range =
                    seg.p_offset as usize..seg.p_offset as usize + seg.p_filesz as usize;
                let slice_range = (seg.p_vaddr as usize).page_offset()
                    ..(seg.p_vaddr as usize).page_offset() + seg.p_filesz as usize;

                start[slice_range].copy_from_slice(&elf_data[elf_range])
            }

            new_pages
        };

        let perms = if seg.p_flags == PF_W | PF_R {
            MappingType::USER_DATA
        } else if seg.p_flags == PF_X | PF_R {
            MappingType::USER_TEXT
        } else if seg.p_flags == PF_R {
            MappingType::USER_DATA_RO
        } else {
            panic!("Unknown elf permissions");
        };

        secs.push(Segment {
            va: virt_range,
            pa: phys_range,
            tp: perms,
        });
    }

    Some(ElfData {
        regions: secs,
        ep: (elf.ehdr.e_entry as usize).into(),
    })
}
