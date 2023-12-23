#![no_std]

pub mod syscalls;
pub mod stdio;
pub mod elf;

pub use rustrt::*;

#[cfg(target_arch = "aarch64")]
mod syscalls_aarch64;

// extern crate alloc;
