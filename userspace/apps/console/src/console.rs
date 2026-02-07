use super::bindings_Serial::Serial;
use super::bindings_Vfs::{Directory, File, Vfs};
use alloc::format;
use alloc::{string::String, vec::Vec};
use hal::address::VirtualAddress;
use libc::vmm::vms::vms;
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

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

    async fn read_until_newline(&self) -> String {
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

    async fn ls(&self, _path: &str) -> String {
        let root = self.vfs.Root().await.unwrap().handle;
        let root = Directory::new(unsafe { Port::new(root) });
        let dir = root.List().await.unwrap();

        dir.entries
            .into_iter()
            .map(|x| format!("{}", x.name))
            .collect::<Vec<_>>()
            .join(" ")
    }

    async fn create(&self, name: &str) -> Result<(), ErrorType> {
        let root = self.vfs.Root().await.unwrap().handle;
        let root = Directory::new(unsafe { Port::new(root) });

        root.CreateFile(name.try_into().unwrap()).await?;
        Ok(())
    }

    async fn write(&self, name: &str, data: &str) -> Result<(), ErrorType> {
        let root = self.vfs.Root().await.unwrap().handle;
        let root = Directory::new(unsafe { Port::new(root) });

        let res = root.CreateFile(name.try_into().unwrap()).await?;
        let file = File::new(unsafe { Port::new(res.handle) });
        let vmo = vms().create_vm_object(data.len(), MappingType::Data)?;
        let mut buf = vms().map_vm_object(&vmo, None, MappingType::Data)?;
        let buf = unsafe { buf.as_slice_mut(data.len()) };

        buf.copy_from_slice(data.as_bytes());

        file.Write(0, buf.len(), vmo.handle()).await?;
        Ok(())
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
                    "ls" => {
                        let dir = parts.next().unwrap_or("");
                        let res = self.ls(dir).await;

                        self.put_str(alloc::format!("{res}\n")).await;
                    }
                    "file_create" => {
                        if let Some(name) = parts.next() {
                            self.create(name).await.unwrap();
                        } else {
                            self.put_str(alloc::format!("Please provide a name\n"))
                                .await
                        }
                    }
                    "append_write" => {
                        if let Some(name) = parts.next() && let Some(text) = parts.next() {
                            self.write(name, text).await.unwrap();
                        } else {
                            self.put_str(alloc::format!("Please provide a name\n"))
                                .await
                        }
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
