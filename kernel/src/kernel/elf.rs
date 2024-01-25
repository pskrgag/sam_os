use alloc::vec::Vec;
use core::mem::size_of;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;
use crate::mm::allocators::page_alloc::page_allocator;
use rtl::arch::PAGE_SIZE;

const EI_NIDENT: usize = 16;
const ELF_MAGIC: [u8; 4] = [0x7f, 'E' as u8, 'L' as u8, 'F' as u8];

#[allow(non_camel_case_types)]
type Elf64_Addr = u64;
#[allow(non_camel_case_types)]
type Elf64_Half = u16;
#[allow(non_camel_case_types)]
type Elf64_SHalf = i16;
#[allow(non_camel_case_types)]
type Elf64_Off = u64;
#[allow(non_camel_case_types)]
type Elf64_Sword = i32;
#[allow(non_camel_case_types)]
type Elf64_Word = u32;
#[allow(non_camel_case_types)]
type Elf64_Xword = u64;
#[allow(non_camel_case_types)]
type Elf64_Sxword = i64;

const ET_EXEC: Elf64_Half = 2;
const ELFCLASS64: u8 = 2;
const EM_AARCH64: Elf64_Half = 183;

const PT_LOAD: Elf64_Word = 1;

const PF_R: u32 = 0x4;
const PF_W: u32 = 0x2;
const PF_X: u32 = 0x1;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct ElfHeader {
    e_ident: [u8; EI_NIDENT],
    e_type: Elf64_Half,
    e_machine: Elf64_Half,
    e_version: Elf64_Word,
    e_entry: Elf64_Addr,
    e_phoff: Elf64_Off,
    e_shoff: Elf64_Off,
    e_flags: Elf64_Word,
    e_ehsize: Elf64_Half,
    e_phentsize: Elf64_Half,
    e_phnum: Elf64_Half,
    e_shentsize: Elf64_Half,
    e_shnum: Elf64_Half,
    e_shstrndx: Elf64_Half,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct ElfIdent {
    magic: [u8; 4],
    class: u8,
    data: u8,
    version: u8,
    os_abi: u8,
    os_abi_ver: u8,
    pad: [u8; 7],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct ElfPhdr {
    p_type: Elf64_Word,
    p_flags: Elf64_Word,
    p_offset: Elf64_Off,
    p_vaddr: Elf64_Addr,
    p_paddr: Elf64_Addr,
    p_filesz: Elf64_Xword,
    p_memsz: Elf64_Xword,
    p_align: Elf64_Xword,
}

#[derive(Debug)]
pub struct ElfData {
    pub regions: Vec<(MemRange<VirtAddr>, MemRange<PhysAddr>, MappingType)>,
    pub ep: VirtAddr,
}

fn read_data<T: Sized + Copy>(data: &mut &[u8]) -> T {
    unsafe {
        let ptr = (*data).as_ptr() as *const T;

        *data = &data[size_of::<T>()..];

        *ptr
    }
}

fn check_ident(ident: &[u8; 16]) -> Option<()> {
    let ident: ElfIdent = unsafe { core::ptr::read(ident.as_ptr() as *const _) };

    match ident.magic {
        ELF_MAGIC => Some(()),
        _ => None,
    }?;

    match ident.class {
        ELFCLASS64 => Some(()),
        _ => None,
    }?;

    match ident.class {
        ELFCLASS64 => Some(()),
        _ => None,
    }?;

    Some(())
}

fn check_header(data: &mut &[u8]) -> Option<ElfHeader> {
    let header = read_data::<ElfHeader>(data);

    check_ident(&header.e_ident)?;

    match header.e_type {
        ET_EXEC => Some(()),
        _ => None,
    }?;

    match header.e_machine {
        EM_AARCH64 => Some(()),
        _ => None,
    }?;

    // #[allow(unaligned_references)]
    // unsafe {
    //     println!(
    //         "Entry point 0x{:x}",
    //         (&header.e_entry as *const u64).read_unaligned()
    //     );
    // }

    Some(header)
}

fn flags_to_mt(flags: Elf64_Word) -> MappingType {
    if flags & PF_W != 0 {
        MappingType::USER_DATA
    } else if flags & PF_X != 0 {
        MappingType::USER_TEXT
    } else {
        MappingType::USER_DATA_RO
    }
}

// TODO: that shit needs rework
fn parse_program_headers(
    data: &mut &[u8],
    header: &ElfHeader,
) -> Option<Vec<(MemRange<VirtAddr>, MemRange<PhysAddr>, MappingType)>> {
    let mut vec = Vec::new();
    let mut data = &data[header.e_phoff as usize - core::mem::size_of::<ElfHeader>()..];

    // I have no time to refactor it. Leaving as is for now, since i test IPC
    let base_pa = PhysAddr::from(VirtAddr::from_raw((*data).as_ptr()))
        - PhysAddr::from(core::mem::size_of::<ElfHeader>());

    assert!(base_pa.is_page_aligned());

    for _ in 0..header.e_phnum {
        let pheader = read_data::<ElfPhdr>(&mut data);

        if pheader.p_type != PT_LOAD {
            continue;
        }

        let size = *(pheader.p_memsz as usize + pheader.p_vaddr as usize
            - *(pheader.p_vaddr as usize).round_down_page())
        .round_up_page();

        let p = pheader.p_vaddr;
        let o = pheader.p_offset;

        // Handle bss properly
        let p_range = if pheader.p_memsz != pheader.p_filesz {
            let p = page_allocator().alloc(*(pheader.p_memsz as usize).round_up_page() / PAGE_SIZE).unwrap();
            MemRange::new(p, pheader.p_memsz as usize)
        } else {
            MemRange::new(
                *(base_pa + PhysAddr::from(pheader.p_offset as usize)).round_down_page(),
                size,
            )
        };

        vec.push((
            MemRange::new(
                *VirtAddr::from(pheader.p_vaddr as usize).round_down_page(),
                size,
            ),
            p_range,
            flags_to_mt(pheader.p_flags),
        ));
    }

    Some(vec)
}

pub fn parse_elf(mut data: &[u8]) -> Option<ElfData> {
    let header = check_header(&mut data)?;
    let secs = parse_program_headers(&mut data, &header)?;

    Some(ElfData {
        regions: secs,
        ep: (header.e_entry as usize).into(),
    })
}
