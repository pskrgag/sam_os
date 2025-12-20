use crate::drivers::uart;
use core::fmt::{self, Write};
use log::{LevelFilter, Log, Metadata, Record};

fn _print(args: fmt::Arguments) {
    uart::uart()
        .write_fmt(args)
        .expect("Failed to write to UART");
}

pub fn print_str<S: AsRef<str>>(s: S) {
    // Must not fail
    uart::uart().write_str(s.as_ref()).unwrap()
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // It's always enabled
        true
    }

    fn log(&self, record: &Record) {
        let current_time = crate::sched::timer::time_since_start();
        let cpu = crate::arch::cpuid::current_cpu();

        _print(format_args!(
            "[{}.{:06}] [{cpu}] {}",
            current_time.as_secs(),
            current_time.subsec_micros(),
            record.args()
        ));
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;

    log::set_logger(&LOGGER).expect("Failed to setup logger");
    log::set_max_level(LevelFilter::Debug)
}
