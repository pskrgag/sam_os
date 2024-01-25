use super::vm_object::VmObject;
use crate::kernel::object::handle::Handle;
use crate::kernel::sched::current;
use crate::mm::paging::page_table::MmError;
use crate::mm::vms::VmsInner;
use alloc::sync::Arc;
use object_lib::object;
use rtl::error::ErrorType;
use rtl::vmm::{types::*, MappingType};
use qrwlock::RwLock;

#[derive(object)]
pub struct Vms {
    inner: RwLock<VmsInner>,
}

impl Vms {
    pub fn new_user() -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(VmsInner::new_user()),
        })
    }

    pub fn vm_map(
        &self,
        v: MemRange<VirtAddr>,
        p: MemRange<PhysAddr>,
        tp: MappingType,
    ) -> Result<VirtAddr, MmError> {
        let mut inner = self.inner.write();

        assert!(v.start().is_page_aligned());
        assert!(p.start().is_page_aligned());
        assert!(p.size().is_page_aligned());
        assert!(v.size().is_page_aligned());

        inner.vm_map(v, p, tp)
    }

    pub fn vm_allocate(&self, size: usize, tp: MappingType) -> Result<VirtAddr, ()> {
        let mut inner = self.inner.write();
        let res = inner.vm_allocate(size, tp)?;

        assert!(res.is_page_aligned());
        Ok(res)
    }

    pub fn base(&self) -> PhysAddr {
        let inner = self.inner.read();
        inner.ttbr0().unwrap()
    }

    fn do_invoke(&self, args: &[usize]) -> Result<usize, ErrorType> {
        use rtl::objects::vms::VmsInvoke;

        match VmsInvoke::from_bits(args[0]).ok_or(ErrorType::NO_OPERATION)? {
            VmsInvoke::ALLOCATE => match self.vm_allocate(args[1], args[2].into()) {
                Ok(v) => Ok(v.into()),
                Err(_) => Err(ErrorType::INVALID_ARGUMENT),
            },
            VmsInvoke::CREATE_VMO => {
                let range = unsafe { core::slice::from_raw_parts(args[1] as *const u8, args[2]) };
                // ToDo: use proper use-copy API
                let vmo = VmObject::from_buffer(range, args[3].into(), args[4].into())
                    .ok_or(ErrorType::NO_MEMORY)?;

                let task = current().unwrap().task();
                let mut table = task.handle_table();
                let handle = Handle::new(vmo.clone());
                let ret = handle.as_raw();

                table.add(handle);

                Ok(ret)
            }
            VmsInvoke::MAP_VMO => {
                let task = current().unwrap().task();
                let table = task.handle_table();

                let vmo = table
                    .find::<VmObject>(args[1])
                    .ok_or(ErrorType::INVALID_ARGUMENT)?;
                let ranges = vmo.as_ranges();

                self.vm_map(ranges.0, ranges.1, vmo.mapping_type()).unwrap();

                Ok(0)
            }
            VmsInvoke::MAP_PHYS => {
                let pa: PhysAddr = args[1].into();
                let size = args[2];
                let mut inner = self.inner.write();

                let range = inner.free_range(size).ok_or(ErrorType::NO_MEMORY)?;

                let va = inner
                    .vm_map(range, MemRange::new(pa, size), MappingType::USER_DEVICE)
                    .unwrap();

                Ok(va.into())
            }
            _ => Err(ErrorType::NO_OPERATION),
        }
    }
}
