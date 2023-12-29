use crate::elf::Elf;
use crate::factory::factory;
use crate::syscalls::Syscall;
use crate::vmm::vms::vms;
use crate::vmm::vms::Vms;
use alloc::string::String;
use alloc::vec::Vec;
use rtl::handle::Handle;
use rtl::objects::task::TaskInvoke;
use rtl::vmm::types::VirtAddr;

pub struct Task {
    name: String,
    h: Handle,
    ep: VirtAddr,
}

impl Task {
    pub fn new(h: Handle, name: String) -> Self {
        Self { h, name, ep: 0.into() }
    }

    pub fn set_ep(&mut self, ep: VirtAddr) {
        self.ep = ep;
    }

    pub fn create_from_elf1(elf_data: &[u8], name: String) -> Option<Self> {
        let elf = Elf::new(elf_data)?;
        let ph = elf.program_headers()?;
        let mut h = Vec::with_capacity(ph.len());

        for i in ph {
            let vm = vms().create_vm_object(
                elf.program_header_to_data(i)?,
                Elf::program_header_to_mapping_type(i),
                VirtAddr::from(i.p_vaddr as usize),
            )?;

            h.push(vm);
        }

        let mut new_task = factory().create_task(name.as_str())?;
        let vms = new_task.vms()?;

        for i in h {
            vms.map_vm_object(&i)?;
        }

        new_task.set_ep(elf.entry_point());

        Some(new_task)
    }

    pub fn start(&mut self) -> Option<()> {
        Syscall::invoke(self.h, TaskInvoke::START.bits(), &[self.ep.into()]).ok()?;
        Some(())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn vms(&self) -> Option<Vms> {
        Some(Vms::new(
            Syscall::invoke(self.h, TaskInvoke::GET_VMS.bits(), &[]).ok()?,
        ))
    }
}
