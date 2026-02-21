use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::Directory;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use rokio::port::Port;

struct Ls;

#[async_trait::async_trait]
impl Command for Ls {
    fn name(&self) -> &str {
        "ls"
    }

    // TODO: actually walk the dir
    async fn run(&self, _args: Vec<&str>, env: Enviroment<'async_trait>) -> Result<String, String> {
        let root = env.vfs.Root().await.unwrap().handle;
        let root = Directory::new(unsafe { Port::new(root) });
        let dir = root.List().await.unwrap();

        Ok(dir
            .entries
            .into_iter()
            .map(|x| alloc::format!("{}", x.name))
            .collect::<Vec<_>>()
            .join(" "))
    }
}

#[linkme::distributed_slice(COMMANDS)]
static LS: &dyn Command = &Ls;
