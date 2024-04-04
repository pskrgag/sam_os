#![no_std]

extern crate alloc;

pub mod allocator;
pub mod elf;
pub mod factory;
pub mod port;
pub mod stdio;
pub mod syscalls;
pub mod task;
pub mod vmm;

pub use rustrt::*;

#[cfg(target_arch = "aarch64")]
mod syscalls_aarch64;

#[cfg(target_arch = "x86_64")]
mod syscalls_x86_64;

pub fn init() -> Option<()> {
    allocator::slab::init()?;
    Some(())
}
