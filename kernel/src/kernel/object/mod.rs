use core::any::Any;

pub mod handle;
pub mod handle_table;

// All exposed kernel objects must be derived from this trait
pub trait KernelObject { 
    fn as_any(&self) -> &dyn Any;
}
