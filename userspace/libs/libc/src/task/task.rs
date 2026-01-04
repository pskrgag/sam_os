use super::Manifest;
use crate::elf::Elf;
use crate::factory::factory;
use crate::handle::Handle;
use crate::syscalls::Syscall;
use crate::vmm::vms::Vms;
use crate::vmm::vms::vms;
use alloc::string::String;
use alloc::vec::Vec;
use hal::address::{Address, VirtAddr, VirtualAddress};
use hal::arch::{PAGE_MASK, PAGE_SIZE};
use postcard::from_bytes;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub struct Task {
    name: String,
    h: Handle,
    ep: VirtAddr,
    manifest: Option<Manifest>,
}

impl Task {
    pub fn new(h: Handle, name: String) -> Self {
        Self {
            h,
            name,
            ep: 0.into(),
            manifest: None,
        }
    }

    pub fn set_ep(&mut self, ep: VirtAddr) {
        self.ep = ep;
    }

    fn parse_manifest(elf: &Elf) -> Result<Manifest, ErrorType> {
        let manifest = elf.section_data(".manifest").ok_or(ErrorType::NotFound)?;

        from_bytes(manifest).map_err(|_| ErrorType::InternalError)
    }

    pub fn manifest(&self) -> &Option<Manifest> {
        &self.manifest
    }

    pub fn create_from_elf(elf_data: &[u8], name: String) -> Result<Self, ErrorType> {
        let elf = Elf::new(elf_data).ok_or(ErrorType::InvalidArgument)?;
        let ph = elf.program_headers().ok_or(ErrorType::InvalidArgument)?;
        let mut h = Vec::with_capacity(ph.len());
        let manifest = Self::parse_manifest(&elf)?;

        for phdr in ph {
            let load_addr = VirtAddr::from(phdr.p_vaddr as usize);
            let to_allocate = ((load_addr.bits() + phdr.p_memsz as usize + PAGE_SIZE) & !PAGE_MASK)
                - (load_addr.bits() & !PAGE_MASK);

            let vm = if phdr.p_filesz != 0 {
                let res = vms().create_vm_object(to_allocate, MappingType::Rwx)?;

                unsafe {
                    // TODO: unmap
                    let mut va = vms().map_vm_object(&res, None, MappingType::Data)?;
                    let slice = va.as_slice_at_offset_mut::<u8>(
                        phdr.p_filesz as usize,
                        phdr.p_vaddr as usize & PAGE_MASK,
                    );

                    slice.copy_from_slice(elf.program_header_to_data(phdr));
                }

                res
            } else {
                vms().create_vm_object(to_allocate, Elf::program_header_to_mapping_type(phdr))?
            };

            h.push((vm, load_addr, Elf::program_header_to_mapping_type(phdr)));
        }

        let mut new_task = factory().create_task(name.as_str())?;
        let vms = new_task.vms().unwrap();

        for (vmo, load, tp) in h {
            let mut load = load.bits();

            load &= !PAGE_MASK;
            vms.map_vm_object(&vmo, Some(VirtAddr::from_bits(load)), tp)
                .unwrap();
        }

        new_task.set_ep(elf.entry_point());
        new_task.manifest = Some(manifest);
        Ok(new_task)
    }

    pub fn start(&mut self, h: &Handle) -> Option<()> {
        Syscall::task_start(&self.h, self.ep, h).ok()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn vms(&self) -> Option<Vms> {
        Some(Vms::new(Syscall::task_get_vms(&self.h).ok()?))
    }
}
