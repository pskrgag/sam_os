use core::fmt::{self, Write};

pub struct Logger {
    backend: Option<*mut dyn Write>,
}

unsafe impl Send for Logger {}

static mut LOGGER: Logger = Logger { backend: None };

pub fn register_logger(backend: *mut dyn Write) {
    unsafe {
        LOGGER = Logger {
            backend: Some(backend),
        }
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    unsafe { (&mut *LOGGER.backend.unwrap()).write_fmt(args).unwrap() };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        ($crate::log::_print(format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($format:expr) => ({
        $crate::log::_print(format_args!(concat!("[LOADER] ", $format, "\n"),));
    });
    ($format:expr, $($arg:tt)*) => ({
        $crate::log::_print(format_args!(concat!("[LOADER] ", $format, "\n"), $($arg)*));
    })
}
