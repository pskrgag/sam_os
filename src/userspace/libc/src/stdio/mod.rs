#![macro_use]

mod log_syscall;

use core::fmt;
use core::fmt::Write;

use log_syscall::log;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    log()
        .write_fmt(args)
        .expect("Failed to write to standart log");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (libc::stdio::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($format:expr) => ({
        libc::stdio::_print(format_args!(
                concat!("{} :: ", $format, "\n"), env!("CARGO_PKG_NAME")
            ));
    });
    ($format:expr, $($arg:tt)*) => ({
        libc::stdio::_print(format_args!(
                concat!("{} :: ", $format, "\n"),
                env!("CARGO_PKG_NAME"),
                $($arg)*
        ));
    })
}

#[macro_export]
macro_rules! println_libc {
    () => (print!("\n"));
    ($fmt:expr, $($args:tt)*) => ({
        $crate::stdio::_print(format_args!(concat!($fmt, "\n"), $($args)*));
    })
}

#[cfg(feature = "verbose")]
#[allow(unused_macros)]
macro_rules! println_libc_verbose {
    () => (print!("\n"));
    ($fmt:expr, $($args:tt)*) => ({
        $crate::stdio::_print(format_args!(concat!($fmt, "\n"), $($args)*));
    })
}

#[cfg(not(feature = "verbose"))]
#[allow(unused_macros)]
macro_rules! println_libc_verbose {
    () => {
        print!("\n")
    };
    ($($arg:tt)*) => {{}};
}
