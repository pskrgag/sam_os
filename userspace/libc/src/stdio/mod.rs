#![macro_use]

mod log_syscall;

use core::fmt;
use core::fmt::Write;

use log_syscall::log;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    log().write_fmt(args).expect("Failed to write to UART");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (libc::stdio::_print(format_args!($($arg)*)));
}

/// Macro similar to [std](https://doc.rust-lang.org/src/std/macros.rs.html)
/// but for writing into kernel-specific output (UART or QEMU console).
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        libc::stdio::_print(format_args_nl!($($arg)*));
    })
}

macro_rules! println_libc {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        crate::stdio::_print(format_args_nl!($($arg)*));
    })
}

#[cfg(feature = "verbose")]
#[allow(unused_macros)]
macro_rules! println_libc_verbose {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        crate::stdio::_print(format_args_nl!($($arg)*));
    })
}

#[cfg(not(feature = "verbose"))]
#[allow(unused_macros)]
macro_rules! println_libc_verbose {
    () => (print!("\n"));
    ($($arg:tt)*) => ({ })
}
