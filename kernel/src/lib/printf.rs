#![macro_use]

use core::fmt;
use core::fmt::Write;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use crate::drivers::uart;

    uart::uart().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::lib::printf::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        let guard = crate::drivers::uart::UART_LOCK.lock_irqsave();
        print!("[{:.10}] ", $crate::arch::time_since_start());
        print!("[CPU{}] ", $crate::arch::cpuid::current_cpu());
        $crate::lib::printf::_print(format_args_nl!($($arg)*));
        drop(guard);
    })
}

//#[cfg(debug_assertions)]
macro_rules! dbg {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::lib::printf::_print(format_args_nl!($($arg)*));
    })
}
