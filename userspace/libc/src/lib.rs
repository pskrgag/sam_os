#![feature(format_args_nl)]
#![no_std]

extern crate alloc;

pub mod syscalls;
pub mod stdio;
pub mod elf;
pub mod allocator;
pub mod task;
pub mod vmm;
pub mod factory;
pub mod port;

pub use rustrt::*;

#[cfg(target_arch = "aarch64")]
mod syscalls_aarch64;

pub fn init() -> Option<()> {
    println_libc!("Started libc init");
    allocator::slab::init()?;
    Some(())
}
