use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::{Directory, File};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use libc::vmm::vms::vms;
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;
use hal::address::VirtualAddress;

struct Cat;

impl Cat {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        let root = env.vfs.Root().await.unwrap().handle;
        let root = Directory::new(unsafe { Port::new(root) });

        let file = root.OpenFile(args[0].try_into().unwrap()).await?;
        let file = File::new(unsafe { Port::new(file.handle) });
        let vmo = vms().create_vm_object(1 << 12, MappingType::Data)?;

        let mut resulting_data = String::new();

        loop {
            let read = file.Read(0, 1 << 12, vmo.handle()).await?.read;
            let buf = vms().map_vm_object(&vmo, None, MappingType::Data)?;
            let buf = unsafe { buf.as_slice(read) };

            resulting_data.push_str(core::str::from_utf8(buf).unwrap());
            if read < 1 << 12 {
                break;
            }
        }

        Ok(resulting_data)
    }
}

#[async_trait::async_trait]
impl Command for Cat {
    fn name(&self) -> &str {
        "cat"
    }

    // TODO: actually walk the dir
    async fn run(&self, args: Vec<&str>, env: Enviroment<'async_trait>) -> Result<String, String> {
        match self.run_internal(args, env).await {
            Ok(s) => Ok(s),
            Err(err) => {
                let s: &str = err.into();

                Err(String::from(s))
            }
        }
    }
}

#[linkme::distributed_slice(COMMANDS)]
static CAT: &dyn Command = &Cat;
