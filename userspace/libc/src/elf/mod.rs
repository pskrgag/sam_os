use elf::ElfBytes;
use elf::segment::ProgramHeader;
use elf::endian::LittleEndian;
use elf::abi::PT_LOAD;

use alloc::vec::Vec;

// ToDo: support any endian?
pub struct Elf<'a> {
    elf_data: ElfBytes<'a, LittleEndian>,
}

impl<'a> Elf<'a> {
    pub fn new(raw_data: &'a [u8]) -> Option<Self> {
        Some(Self {
            elf_data: ElfBytes::<LittleEndian>::minimal_parse(raw_data).ok()?,
        })
    }

    pub fn program_headers(&self) -> Option<ProgramHeader> {
        // let a: Vec<_> = self.elf_data.segments().unwrap()
        //         .iter()
        //         .filter(|phdr|{phdr.p_type == PT_LOAD})
        //         .collect();

        todo!();
    }
}
