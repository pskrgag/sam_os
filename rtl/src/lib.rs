#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(const_mut_refs)]

extern crate static_assertions;

pub mod arch;
pub mod cpio;
pub mod error;
pub mod handle;
pub mod locking;
pub mod syscalls;
pub mod vmm;

pub mod misc;

pub mod objects;
