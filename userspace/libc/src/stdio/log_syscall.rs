use crate::syscalls::*;
use core::fmt;

pub struct Log;

impl fmt::Write for Log {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Syscall::debug_write(s);
        Ok(())
    }
}

pub fn log() -> Log {
    Log {}
}
