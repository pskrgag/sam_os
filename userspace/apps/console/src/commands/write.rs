use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::{Directory, File};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use hal::address::VirtualAddress;
use libc::vmm::vms::vms;
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

struct Write;

impl Write {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        let root = env.vfs.Root().await.unwrap().handle;
        let root = Directory::new(unsafe { Port::new(root) });
        let data = args[1];
        let name = args[0];

        let res = root.CreateFile(name.try_into().unwrap()).await?;
        let file = File::new(unsafe { Port::new(res.handle) });
        let vmo = vms().create_vm_object(data.len(), MappingType::Data)?;
        let mut buf = vms().map_vm_object(&vmo, None, MappingType::Data)?;
        let buf = unsafe { buf.as_slice_mut(data.len()) };

        buf.copy_from_slice(data.as_bytes());

        file.Write(0, buf.len(), vmo.handle()).await?;
        Ok(String::new())
    }
}

#[async_trait::async_trait]
impl Command for Write {
    fn name(&self) -> &str {
        "write"
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
static WRITE: &dyn Command = &Write;
