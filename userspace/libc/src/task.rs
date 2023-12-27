use crate::elf::Elf;
use crate::syscalls::Syscall;
use crate::vmm::vm_object::VmObject;
use alloc::vec::Vec;
use rtl::handle::Handle;
use rtl::vmm::types::VirtAddr;

pub struct Task {
    h: Handle,
}

impl Task {
    pub fn create_from_elf(elf_data: &[u8]) -> Option<Self> {
        let elf = Elf::new(elf_data)?;
        let ph = elf.program_headers()?;
        let mut h = Vec::with_capacity(ph.len());

        for i in ph {
            let vm = VmObject::new_from_buf(
                elf.program_header_to_data(i)?,
                Elf::program_header_to_mapping_type(i),
                VirtAddr::from(i.p_vaddr as usize),
            )?;

            h.push(vm);
        }

        let new_task = Syscall::task_create_from_vmo(
            "tmp",
            h.iter().map(|x| x.handle()).collect::<Vec<_>>().as_slice(),
            elf.entry_point(),
        ).ok()?;

        Some(Self { h: new_task })
    }

    pub fn start(&mut self) -> Option<()> {
        Syscall::task_start(self.h).ok()?;
        Some(())
    }
}
