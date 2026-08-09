use super::{COMMANDS, Command, Enviroment};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use fs::{dir::OpenOptions, path::Path};
use rtl::error::ErrorType;

struct Ls;

impl Ls {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        _env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        let entries = if args.is_empty() {
            fs::cwd().list().await?
        } else {
            let path = Path::new(&args[0]);
            fs::cwd()
                .open_dir(&path, OpenOptions { create: false })
                .await?
                .list()
                .await?
        };

        Ok(entries
            .into_iter()
            .map(|x| alloc::format!("{}{}", x.name, if x.is_directory { "/" } else { "" }))
            .collect::<Vec<_>>()
            .join(" "))
    }
}

#[async_trait::async_trait]
impl Command for Ls {
    fn name(&self) -> &str {
        "ls"
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
static LS: &dyn Command = &Ls;
