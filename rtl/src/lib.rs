#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(const_mut_refs)]

extern crate static_assertions;

pub mod syscalls;
pub mod vmm;
pub mod error;
pub mod arch;
pub mod locking;
