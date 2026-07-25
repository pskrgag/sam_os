use super::bindings_Serial::Serial;
use super::bindings_Vfs::Vfs;
use crate::commands::{commands, Enviroment};
use crate::cwd::Cwd;
use alloc::{string::String, vec::Vec};
use heapless::String as HLString;

pub struct Console {
    backend: Serial,
    vfs: Vfs,
}

impl Console {
    pub fn new(backend: Serial, vfs: Vfs) -> Self {
        Self { backend, vfs }
    }

    async fn put_str<S: AsRef<str>>(&self, s: S) {
        self.backend
            .Put(s.as_ref().try_into().unwrap())
            .await
            .unwrap();
    }

    async fn put_byte(&self, byte: u8) {
        let mut s = HLString::new();

        // this will never happen
        s.push(byte as char).unwrap();
        self.backend.Put(s.try_into().unwrap()).await.unwrap();
    }

    async fn read_until_newline(&self) -> String {
        let mut res = String::new();

        loop {
            let new = self.backend.GetByte().await.unwrap();

            match new.byte {
                b'\r' => {
                    self.put_str("\n").await;
                    break res;
                }
                0x08 | 0x7f => {
                    res.pop();
                    self.put_str("\x08 \x08").await;
                    continue;
                }
                s => {
                    self.put_byte(s).await;
                }
            }

            res.push(new.byte as char);
        }
    }

    pub async fn serve(self) {
        let mut cwd = Cwd::root(&self.vfs).await.unwrap();

        loop {
            self.put_str(alloc::format!("{} > ", cwd.name())).await;

            let cmd = self.read_until_newline().await;
            if cmd.is_empty() {
                continue;
            }

            let mut parts = cmd.split_whitespace();
            let cmd_name = parts.next();

            if let Some(cmd_name) = cmd_name {
                let args: Vec<_> = parts.collect();
                let mut executed = false;

                for cmd in commands() {
                    if cmd.name() == cmd_name {
                        let res = match cmd.run(args, Enviroment { cwd: &mut cwd }).await {
                            Err(e) => e,
                            Ok(e) => e,
                        };

                        if !res.is_empty() {
                            self.put_str(res).await;
                            self.put_str("\n").await;
                        }

                        executed = true;
                        break;
                    }
                }

                if !executed {
                    self.put_str(alloc::format!("Unknown command '{cmd_name}'\n"))
                        .await
                }
            } else {
                self.put_str("Failed to parse command\n").await;
            }
        }
    }
}
