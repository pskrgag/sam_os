use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::Directory;
use crate::cwd::Cwd;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use fs::path::Path;
use rokio::port::Port;
use rtl::error::ErrorType;

struct Cd;

impl Cd {
    fn updated_path(current: &str, requested: &str) -> String {
        let current = Path::new(&current);
        let mut components = current
            .components()
            .map(String::from)
            .collect::<Vec<_>>();

        let requested = Path::new(&requested);
        for component in requested.components() {
            match component {
                "." => {}
                ".." => {
                    components.pop();
                }
                component => components.push(String::from(component)),
            }
        }

        if components.is_empty() {
            String::from("/")
        } else {
            alloc::format!("/{}", components.join("/"))
        }
    }

    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.is_empty() {
            return Err(ErrorType::InvalidArgument);
        }

        let path = Self::updated_path(env.cwd.name(), args[0]);
        let current = &**env.cwd;
        let dir = current
            .OpenDir(args[0].try_into().map_err(|_| ErrorType::BufferTooBig)?, 0)
            .await?;

        *env.cwd = Cwd::new(Directory::new(unsafe { Port::new(dir.handle) }), path);
        Ok(String::new())
    }
}

#[async_trait::async_trait]
impl Command for Cd {
    fn name(&self) -> &str {
        "cd"
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
static CD: &dyn Command = &Cd;
