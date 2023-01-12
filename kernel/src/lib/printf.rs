#![macro_use]

use core::fmt;
use core::fmt::Write;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use crate::drivers::uart;

    uart::uart().write_fmt(args).expect("Failed to write to UART");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::lib::printf::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($format:expr) => ({
        $crate::lib::printf::_print(format_args_nl!(
                concat!("[{:.10}] [CPU{}] ", $format),
                $crate::arch::time_since_start(),
                $crate::arch::cpuid::current_cpu()
            ));
    });
    ($format:expr, $($arg:tt)*) => ({
        $crate::lib::printf::_print(format_args_nl!(
                concat!("[{:.10}] [CPU{}] ", $format),
                $crate::arch::time_since_start(),
                $crate::arch::cpuid::current_cpu(),
                $($arg)*
            ));
    })
}

//#[cfg(debug_assertions)]
macro_rules! dbg {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::lib::printf::_print(format_args_nl!($($arg)*));
    })
}
