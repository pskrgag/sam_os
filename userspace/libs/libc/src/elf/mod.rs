use elf::ElfBytes;
use elf::abi::PT_LOAD;
use elf::endian::LittleEndian;
use elf::segment::ProgramHeader;
use hal::address::VirtAddr;
use rtl::vmm::MappingType;

// const PF_R: u32 = 0x4;
const PF_W: u32 = 0x2;
const PF_X: u32 = 0x1;

use alloc::vec::Vec;

pub struct Elf<'a> {
    raw: &'a [u8],
    elf_data: ElfBytes<'a, LittleEndian>,
}

impl<'a> Elf<'a> {
    pub fn new(raw_data: &'a [u8]) -> Option<Self> {
        Some(Self {
            raw: raw_data,
            elf_data: match ElfBytes::<LittleEndian>::minimal_parse(raw_data) {
                Ok(data) => Some(data),
                Err(err) => {
                    crate::println_libc!("Failed to parse elf file {}", err);
                    None
                }
            }?,
        })
    }

    pub fn program_headers(&self) -> Option<Vec<ProgramHeader>> {
        Some(
            self.elf_data
                .segments()?
                .iter()
                .filter(|phdr| phdr.p_type == PT_LOAD)
                .collect(),
        )
    }

    pub fn entry_point(&self) -> VirtAddr {
        (self.elf_data.ehdr.e_entry as usize).into()
    }

    pub fn program_header_to_data(&self, h: ProgramHeader) -> &'a [u8] {
        &self.raw[h.p_offset as usize..(h.p_offset + h.p_filesz) as usize]
    }

    pub fn program_header_to_mapping_type(h: ProgramHeader) -> MappingType {
        if h.p_flags & PF_W != 0 {
            MappingType::Data
        } else if h.p_flags & PF_X != 0 {
            MappingType::Text
        } else {
            MappingType::RoData
        }
    }

    pub fn section_data(&self, section_name: &str) -> Option<&[u8]> {
        if let Ok((shdrs, strtab)) = self.elf_data.section_headers_with_strtab()
            && let Some(strtab) = strtab
            && let Some(shdrs) = shdrs
        {
            for section in shdrs {
                if let Ok(name) = strtab.get(section.sh_name as usize)
                    && name == section_name
                {
                    return Some(self.elf_data.section_data(&section).unwrap().0);
                }
            }
        }

        None
    }
}
