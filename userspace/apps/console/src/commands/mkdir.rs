use super::{COMMANDS, Command, Enviroment};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use fs::{dir::OpenOptions, path::Path};
use rtl::error::ErrorType;

struct Mkdir;

impl Mkdir {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        _env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.is_empty() {
            return Err(ErrorType::InvalidArgument);
        }

        let path = Path::new(&args[0]);
        fs::cwd()
            .open_dir(&path, OpenOptions { create: true })
            .await?;
        Ok(String::new())
    }
}

#[async_trait::async_trait]
impl Command for Mkdir {
    fn name(&self) -> &str {
        "mkdir"
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
static MKDIR: &dyn Command = &Mkdir;
