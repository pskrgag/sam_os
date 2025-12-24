use super::vma_list::{VmaFlag, VmaList};
use super::vmo::VmObject;
use crate::arch::mm::page_table::switch_context;
use crate::mm::paging::kernel_page_table::kernel_page_table;
use crate::mm::{allocators::page_alloc::page_allocator, paging::page_table::PageTable};
use crate::object::capabilities::{Capability, CapabilityMask};
use crate::object::handle::Handle;
use crate::object::KernelObjectBase;
use crate::sync::Mutex;
use alloc::sync::Arc;
use hal::address::{Address, MemRange, PhysAddr, VirtAddr, VirtualAddress};
use hal::arch::*;
use rtl::error::ErrorType;
use rtl::signal::Signal;
use rtl::vmm::MappingType;

pub struct VmsInner {
    ttbr0: Option<PageTable>,
    vmas: VmaList,
}

impl VmsInner {
    pub fn new_user() -> Option<Self> {
        Some(Self {
            ttbr0: Some(PageTable::new()?),
            vmas: VmaList::new_user(),
        })
    }

    pub fn new_kernel() -> Self {
        Self {
            ttbr0: None,
            vmas: VmaList::new_kernel(),
        }
    }

    pub fn vm_map(
        &mut self,
        v: Option<MemRange<VirtAddr>>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, ErrorType> {
        debug_assert!(p.start().is_page_aligned());
        debug_assert_eq!(p.size().next_multiple_of(PAGE_SIZE), p.size());

        let size = p.size();

        let va = self.vmas.new_vma(
            size,
            v.map(|x| x.start()).map(|x| x.bits()),
            tp,
            VmaFlag::ExternalPages.into(),
        )?;

        self.ttbr0
            .as_mut()
            .unwrap()
            .map(p, MemRange::new(va, size), tp)?;

        Ok(va)
    }

    // ToDo: on-demang allocation of physical memory
    pub fn vm_allocate(&mut self, mut size: usize, tp: MappingType) -> Result<VirtAddr, ErrorType> {
        if !size.next_multiple_of(PAGE_SIZE) == size {
            return Err(ErrorType::InvalidArgument);
        }

        let mut new_va = self.vmas.new_vma(size, None, tp, VmaFlag::None.into())?;
        let ret = new_va;

        while size != 0 {
            let p = if let Some(p) = page_allocator().alloc(1) {
                p
            } else {
                return Err(ErrorType::NoMemory);
            };

            // ToDo: clean up in case of an error
            self.ttbr0
                .as_mut()
                .unwrap_or(&mut kernel_page_table())
                .map(
                    MemRange::new(p, PAGE_SIZE),
                    MemRange::new(new_va, PAGE_SIZE),
                    tp,
                )
                .map_err(|_| ErrorType::NoMemory)?;

            size -= PAGE_SIZE;
            new_va.add(PAGE_SIZE);
        }

        Ok(ret)
    }

    pub fn vm_free(&mut self, range: MemRange<VirtAddr>) -> Result<(), ErrorType> {
        debug_assert!(range.start().is_page_aligned());
        debug_assert_eq!(range.size().next_multiple_of(PAGE_SIZE), range.size());

        self.vmas.free(range)?;

        self.ttbr0
            .as_mut()
            .unwrap_or(&mut kernel_page_table())
            .free(range, |pa| {
                // TODO: check if VMA has ExternalPages flag
                page_allocator().free(pa, 1);
            })
            .expect("Failed to free memory");

        Ok(())
    }

    pub fn ttbr0(&self) -> Option<PhysAddr> {
        self.ttbr0.as_ref().map(|ttbr0| ttbr0.base())
    }

    pub fn translate(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.ttbr0.as_ref()?.translate(va)
    }
}

pub struct Vms {
    inner: Mutex<VmsInner>,
    tt_base: PhysAddr,
    base: KernelObjectBase,
}

crate::kernel_object!(Vms, Signal::None.into());

impl Vms {
    pub fn new_user() -> Option<Arc<Self>> {
        let vms = VmsInner::new_user()?;
        let new = Self {
            tt_base: vms.ttbr0().unwrap(),
            inner: Mutex::new(vms),
            base: KernelObjectBase::new(),
        };

        Arc::try_new(new).ok()
    }

    pub fn new_kernel() -> Option<Arc<Self>> {
        let vms = VmsInner::new_kernel();
        let new = Self {
            tt_base: kernel_page_table().base(),
            inner: Mutex::new(vms),
            base: KernelObjectBase::new(),
        };

        Arc::try_new(new).ok()
    }

    pub fn full_caps() -> CapabilityMask {
        CapabilityMask::from(Capability::MapPhys)
    }

    pub async fn vm_map(
        &self,
        v: Option<MemRange<VirtAddr>>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, ErrorType> {
        let mut inner = self.inner.lock().await?;

        debug_assert!(p.start().is_page_aligned());
        debug_assert_eq!(p.size().next_multiple_of(PAGE_SIZE), p.size());

        inner.vm_map(v, p, tp)
    }

    pub async fn vm_allocate(&self, size: usize, tp: MappingType) -> Result<VirtAddr, ErrorType> {
        let mut inner = self.inner.lock().await?;
        let res = inner.vm_allocate(size, tp)?;

        debug_assert!(res.is_page_aligned());
        Ok(res)
    }

    pub async fn vm_free(&self, base: VirtAddr, size: usize) -> Result<(), ErrorType> {
        let mut inner = self.inner.lock().await?;

        inner
            .vm_free(MemRange::new(base, size))
            .map_err(|_| ErrorType::InvalidArgument)
    }

    pub fn base(&self) -> PhysAddr {
        self.tt_base
    }

    pub fn create_vmo(&self, size: usize, mt: MappingType) -> Result<Handle, ErrorType> {
        let vmo = VmObject::zeroed(size, mt).ok_or(ErrorType::NoMemory)?;

        Ok(Handle::new(vmo, CapabilityMask::any()))
    }

    pub async fn map_phys(&self, pa: PhysAddr, size: usize) -> Result<*mut u8, ErrorType> {
        let mut inner = self.inner.lock().await?;

        let va = inner.vm_map(None, MemRange::new(pa, size), MappingType::Device)?;
        Ok(va.to_raw_mut::<u8>())
    }

    pub fn switch_to(&self) {
        switch_context(self.base());
    }

    pub async fn translate(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.inner.lock().await.ok().and_then(|x| x.translate(va))
    }
}
