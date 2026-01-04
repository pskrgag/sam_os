use super::bindings_Serial::Serial;
use alloc::{string::String, vec::Vec};

pub struct Console {
    backend: Serial,
}

impl Console {
    pub fn new(backend: Serial) -> Self {
        Self { backend }
    }

    pub async fn put_str<S: AsRef<str>>(&self, s: S) {
        self.backend
            .Put(s.as_ref().try_into().unwrap())
            .await
            .unwrap();
    }

    pub async fn read_until_newline(&self) -> String {
        let mut res = String::new();

        loop {
            let new = self.backend.GetByte().await.unwrap();

            if new.byte == b'\r' {
                self.put_str("\n").await;
                break res;
            } else {
                let mut s = String::with_capacity(1);

                s.push(new.byte as char);
                self.put_str(&s).await;
            }

            res.push(new.byte as char);
        }
    }

    pub async fn serve(self) {
        loop {
            self.put_str("> ").await;
            let cmd = self.read_until_newline().await;

            if cmd.is_empty() {
                continue;
            }

            let mut parts = cmd.split_whitespace();
            let cmd_name = parts.next();

            if let Some(cmd_name) = cmd_name {
                match cmd_name {
                    "echo" => {
                        let echo = parts.collect::<Vec<_>>().join(" ");

                        self.put_str(alloc::format!("{echo}\n")).await;
                    }
                    _ => {
                        self.put_str(alloc::format!("Unknown command '{cmd_name}'\n"))
                            .await
                    }
                }
            } else {
                self.put_str("Failed to parse command\n").await;
            }
        }
    }
}
