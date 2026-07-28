use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::File;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use hal::address::VirtualAddress;
use libc::factory::factory;
use libc::vmm::vms::vms;
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

struct Cat;

impl Cat {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.is_empty() {
            return Err(ErrorType::InvalidArgument);
        }

        let file = env.cwd.OpenFile(args[0].try_into().unwrap(), 0).await?;
        let file = File::new(unsafe { Port::new(file.handle) });
        let vmo = factory().create_vm_object(1 << 12, MappingType::Data)?;

        let mut resulting_data = String::new();
        let mut offset = 0;

        loop {
            let read = file.Read(offset, 1 << 12, vmo.handle()).await?.read;
            let buf = vms().map_vm_object(&vmo, None, MappingType::Data)?;
            let buf = unsafe { buf.as_slice(read) };

            resulting_data.push_str(core::str::from_utf8(buf).unwrap());
            offset += read;
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
