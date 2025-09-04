use core::any::Any;

pub mod handle;
pub mod handle_table;

pub mod factory_object;
pub mod port_object;
pub mod task_object;
pub mod thread_object;
pub mod vm_object;
pub mod vms_object;

// All exposed kernel objects must be derived from this trait
pub trait KernelObject {
    fn as_any(&self) -> &dyn Any;
}
