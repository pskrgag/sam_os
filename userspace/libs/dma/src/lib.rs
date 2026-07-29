//! Arch agnostic wrappers for DMA
#![no_std]

use core::marker::PhantomData;
use core::mem::size_of;
use hal::address::{MemRange, PhysAddr, VirtAddr, VirtualAddress};
use libc::factory::factory;
use libc::vmm::vm_object::VmObject;
use libc::vmm::vms::vms;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub struct DmaBuffer<T: Copy> {
    va: MemRange<VirtAddr>,
    pa: MemRange<PhysAddr>,
    vmo: VmObject,
    _pd: PhantomData<T>,
}

impl<T: Copy> DmaBuffer<T> {
    pub fn new(entries: usize) -> Result<Self, ErrorType> {
        let num_bytes = size_of::<T>() * entries;

        let vmo = factory().create_vm_object_contig(num_bytes, MappingType::Device)?;
        let va = vms().map_vm_object(&vmo, None, MappingType::Device)?;
        let pa = vmo.get_phys_info()?;

        Ok(Self {
            va: MemRange::new(va, num_bytes),
            pa: MemRange::new(pa, num_bytes),
            vmo,
            _pd: PhantomData,
        })
    }

    pub fn size(&self) -> usize {
        self.va.size()
    }

    pub fn pa(&self) -> PhysAddr {
        self.pa.start()
    }

    pub fn write(&mut self, idx: usize, val: T) {
        assert!(idx < self.va.size() / size_of::<T>());

        // TODO: here we rely on kernel mapping VMO as uncached NeReGe memory. In future it would be
        // better to do cache maintance in the user-space
        unsafe {
            self.va
                .start()
                .to_raw_mut::<T>()
                .add(idx)
                .write_volatile(val);

            core::arch::asm!("dsb sy");
        }
    }
}

impl<T: Copy> Drop for DmaBuffer<T> {
    fn drop(&mut self) {
        vms()
            .vm_free(self.va.start().to_raw_mut(), self.va.size())
            .unwrap();
    }
}
