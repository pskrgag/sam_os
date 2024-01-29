use elf::abi::PT_LOAD;
use elf::endian::LittleEndian;
use elf::segment::ProgramHeader;
use elf::ElfBytes;
use rtl::vmm::MappingType;
use rtl::vmm::types::VirtAddr;

// const PF_R: u32 = 0x4;
const PF_W: u32 = 0x2;
const PF_X: u32 = 0x1;

use alloc::vec::Vec;

// ToDo: support any endian?
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
                    println_libc!("Failed to parse elf file {}", err);
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

    pub fn program_header_to_data(&self, h: ProgramHeader) -> Option<&'a [u8]> {
        Some(&self.raw[h.p_offset as usize..(h.p_offset + h.p_filesz) as usize])
    }

    pub fn program_header_to_mapping_type(h: ProgramHeader) -> MappingType {
        if h.p_flags & PF_W != 0 {
            MappingType::USER_DATA
        } else if h.p_flags & PF_X != 0 {
            MappingType::USER_TEXT
        } else {
            MappingType::USER_DATA_RO
        }
    }
}
