use super::bindings_Serial::Serial;
use alloc::{string::String, vec::Vec};

pub struct Console {
    backend: Serial,
}

impl Console {
    pub fn new(backend: Serial) -> Self {
        Self { backend }
    }

    pub fn put_str<S: AsRef<str>>(&self, s: S) {
        for byte in s.as_ref().bytes() {
            self.backend.Put(byte).unwrap();
        }
    }

    pub fn read_until_newline(&self) -> String {
        let mut res = String::new();

        loop {
            let new = self.backend.Get().unwrap();

            if new.byte == b'\r' {
                self.backend.Put(b'\n').unwrap();
                break res;
            } else {
                self.backend.Put(new.byte).unwrap();
            }

            res.push(new.byte as char);
        }
    }

    pub fn serve(self) {
        loop {
            self.put_str("> ");
            let cmd = self.read_until_newline();

            if cmd == "" {
                continue;
            }

            let mut parts = cmd.split_whitespace();
            let cmd_name = parts.next();

            if let Some(cmd_name) = cmd_name {
                match cmd_name {
                    "echo" => {
                        let echo = parts.collect::<Vec<_>>().join(" ");

                        self.put_str(alloc::format!("{echo}\n"));
                    }
                    _ => self.put_str(alloc::format!("Unknown command '{cmd_name}'\n")),
                }
            } else {
                self.put_str("Failed to parse command\n");
            }
        }
    }
}

