use super::port_object::Port;
use crate::drivers::irq::IntId;
use crate::irq::IrqObject;
use crate::mm::vmm::vmo::VmObject;
use crate::object::capabilities::{Capability, CapabilityMask};
use crate::object::handle::Handle;
use crate::object::KernelObjectBase;
use crate::sched::current;
use crate::tasks::task::{Task, TaskName};
use alloc::sync::Arc;
use rtl::error::ErrorType;
use rtl::signal::Signal;
use rtl::vmm::MappingType;
use rtl::irq::IrqTrigger;
use spin::Lazy;

pub struct Factory {
    base: KernelObjectBase,
}

crate::kernel_object!(Factory, Signal::None.into());

pub static FACTORY: Lazy<Arc<Factory>> = Lazy::new(|| Factory::new().unwrap());

impl Factory {
    fn new() -> Option<Arc<Factory>> {
        Arc::try_new(Self {
            base: KernelObjectBase::new(),
        })
        .ok()
    }

    pub fn create_task(&self, name: &str) -> Result<Handle, ErrorType> {
        let name = TaskName::try_from(name).map_err(|_| ErrorType::BufferTooBig)?;
        let task = Task::new(name).ok_or(ErrorType::NoMemory)?;
        let handle = Handle::new(task, CapabilityMask::any());

        Ok(handle)
    }

    pub fn create_port(&self) -> Result<Handle, ErrorType> {
        let task = current().task();
        let port = Port::new(task.clone()).ok_or(ErrorType::NoMemory)?;

        Ok(Handle::new(port, Port::full_caps()))
    }

    pub fn create_vmo(&self, size: usize, mt: MappingType) -> Result<Handle, ErrorType> {
        let vmo = VmObject::new(size, mt).ok_or(ErrorType::NoMemory)?;

        Ok(Handle::new(vmo, CapabilityMask::any()))
    }

    pub fn create_vmo_contig(&self, size: usize, mt: MappingType) -> Result<Handle, ErrorType> {
        let vmo = VmObject::new_contig(size, mt).ok_or(ErrorType::NoMemory)?;

        Ok(Handle::new(
            vmo,
            CapabilityMask::from(Capability::GetPhysInfo),
        ))
    }

    pub fn create_irq(&self, num: usize, trigger: IrqTrigger) -> Result<Handle, ErrorType> {
        let num = u32::try_from(num).map_err(|_| ErrorType::InvalidArgument)?;

        if num >= IntId::MAX_SPI_COUNT {
            return Err(ErrorType::InvalidArgument);
        }

        Ok(Handle::new(
            IrqObject::new(IntId::spi(num), trigger)?,
            CapabilityMask::from(Capability::Wait),
        ))
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        panic!("Factory dropped");
    }
}
