#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

pub const USER_AS_START: usize = crate::arch::PAGE_SIZE;
pub const USER_AS_SIZE: usize = USER_AS_END - USER_AS_START + 1;

static_assertions::const_assert!(USER_AS_SIZE.is_multiple_of(PAGE_SIZE));
