use crate::mm::user_buffer::UserPtr;
use crate::tasks::task::init_task;
use elf::{
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
    ElfBytes,
};
use hal::address::*;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub async fn prepare_initial_task(
    prot: &loader_protocol::LoaderArg,
) -> Result<VirtAddr, ErrorType> {
    let elf_data = unsafe {
        core::slice::from_raw_parts(
            prot.init_virt_task_base.0 as *const u8,
            prot.init_virt_task_base.1,
        )
    };
    let elf =
        ElfBytes::<LittleEndian>::minimal_parse(elf_data).expect("Failed to parse kernel elf");
    let task = init_task();
    let vms = task.vms();

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

        let perms = if seg.p_flags == PF_W | PF_R {
            MappingType::Data
        } else if seg.p_flags == PF_X | PF_R {
            MappingType::Text
        } else if seg.p_flags == PF_R {
            MappingType::RoData
        } else {
            panic!("Unknown elf permissions");
        };

        vms.vm_allocate(
            virt_range.size(),
            MappingType::Data,
            Some(virt_range.start()),
        )
        .await?;

        task.with_attached_task(|| {
            if seg.p_filesz != 0 {
                let mut uptr = UserPtr::<u8>::new_array(seg.p_vaddr as *const u8, seg.p_memsz as _);
                let elf_range =
                    seg.p_offset as usize..seg.p_offset as usize + seg.p_filesz as usize;

                uptr.write_array(&elf_data[elf_range]).unwrap();
            }
        });

        vms.vm_protect(virt_range, perms).await?;
    }

    Ok(VirtAddr::from_bits(elf.ehdr.e_entry as usize))
}
