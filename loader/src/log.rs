use core::fmt::{self, Write};
use log::{LevelFilter, Log, Metadata, Record};

struct LoaderLogger {
    backend: Option<*mut dyn Write>,
}

unsafe impl Send for LoaderLogger {}

static mut LOGGER: LoaderLogger = LoaderLogger { backend: None };

pub fn register_logger(backend: *mut dyn Write) {
    unsafe {
        LOGGER = LoaderLogger {
            backend: Some(backend),
        }
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    unsafe { (&mut *LOGGER.backend.unwrap()).write_fmt(args).unwrap() };
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // It's always enabled
        true
    }

    fn log(&self, record: &Record) {
        _print(format_args!("[LOADER] {}", record.args()));
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;

    log::set_logger(&LOGGER).expect("Failed to setup logger");
    log::set_max_level(LevelFilter::Debug)
}
