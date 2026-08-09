use super::{COMMANDS, Command, Enviroment};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use fs::{dir::OpenOptions, path::Path};
use rtl::error::ErrorType;

struct Write;

impl Write {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        _env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.len() < 2 {
            return Err(ErrorType::InvalidArgument);
        }

        let data = args[0];
        let name = args[1];

        let path = Path::new(&name);
        let file = fs::cwd()
            .open_file(&path, OpenOptions { create: true })
            .await?;
        file.write(data.as_bytes()).await?;
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
