use super::{COMMANDS, Command, Enviroment};
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
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.len() == 0 {
            return Err(ErrorType::InvalidArgument);
        }

        let path = Path::new(&args[0]);
        let mut opened = None;

        for comp in path.components() {
            let current = opened.as_ref().unwrap_or(&**env.cwd);
            let dir = current
                .OpenDir(comp.try_into().map_err(|_| ErrorType::BufferTooBig)?, 0)
                .await?;

            opened = Some(Directory::new(unsafe { Port::new(dir.handle) }));
        }

        let dir = opened.ok_or(ErrorType::InvalidArgument)?;
        *env.cwd = Cwd::new(dir, args[0]);
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
