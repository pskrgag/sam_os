#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]

extern crate static_assertions;

pub mod error;
pub mod handle;
pub mod ipc;
pub mod locking;
pub mod misc;
pub mod syscalls;
pub mod vmm;
