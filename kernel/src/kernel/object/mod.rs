use core::any::Any;

pub mod handle;
pub mod handle_table;
pub mod handle_allocator;

// All exposed kernel objects must be derived from this trait
pub trait KernelObject {
    fn as_any(&self) -> &dyn Any;
}

pub fn init() {
    handle_allocator::init();

    println!("Object subsystem initialized");
}
