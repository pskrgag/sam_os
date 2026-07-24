use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::{DirEntryFlagsFlag, Directory};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use fs::path::Path;
use rokio::port::Port;
use rtl::error::ErrorType;

struct Ls;

impl Ls {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        let mut dirs = Vec::new();

        let dir = if args.is_empty() {
            env.cwd
        } else {
            let mut iter = &(**env.cwd);
            let path = Path::new(&args[0]);

            for comp in path.components() {
                let dir = iter
                    .OpenDir(comp.try_into().map_err(|_| ErrorType::BufferTooBig)?, 0)
                    .await?;

                dirs.push(Directory::new(unsafe { Port::new(dir.handle) }));
                iter = dirs.last().unwrap();
            }

            iter
        };

        let entries = dir.List().await?;

        Ok(entries
            .entries
            .into_iter()
            .map(|x| {
                alloc::format!(
                    "{}{}",
                    x.name,
                    if x.flags == DirEntryFlagsFlag::Directory.into() {
                        "/"
                    } else {
                        ""
                    }
                )
            })
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
