#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]

extern crate static_assertions;

pub mod arch;
pub mod cpio;
pub mod error;
pub mod handle;
pub mod ipc;
pub mod locking;
pub mod syscalls;
pub mod vmm;

pub mod uart;

pub mod misc;

pub mod objects;
