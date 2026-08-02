use super::{Command, Enviroment, COMMANDS};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use fs::{dir::OpenOptions, path::Path};
use rtl::error::ErrorType;

struct Cat;

impl Cat {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        _env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.is_empty() {
            return Err(ErrorType::InvalidArgument);
        }

        let path = Path::new(&args[0]);
        let file = fs::cwd()
            .open_file(&path, OpenOptions { create: false })
            .await?;
        let data = file.read_to_end().await?;

        Ok(String::from(core::str::from_utf8(&data).unwrap()))
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
