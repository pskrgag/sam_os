use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::File;
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
        if args.len() < 2 {
            return Err(ErrorType::InvalidArgument);
        }

        let data = args[0];
        let name = args[1];

        let res = env.cwd.OpenFile(name.try_into().unwrap(), 1).await?;
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
