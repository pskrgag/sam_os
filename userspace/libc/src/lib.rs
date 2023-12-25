#![no_std]

extern crate alloc;

pub mod syscalls;
pub mod stdio;
pub mod elf;
pub mod allocator;

pub use rustrt::*;

#[cfg(target_arch = "aarch64")]
mod syscalls_aarch64;

pub fn init() -> Option<()> {
    allocator::slab::init()?;
    Some(())
}
