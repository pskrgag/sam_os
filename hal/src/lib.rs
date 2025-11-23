#![no_std]
///! Common HAL-related things that are shared between kernel user-space and bootloader

// Page structure definition
pub mod page;

// Arch-specific defines
pub mod arch;

// Addresses
pub mod address;
