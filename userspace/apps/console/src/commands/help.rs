use super::{Command, Enviroment, COMMANDS};
use crate::bindings_Vfs::Directory;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use rokio::port::Port;

struct Help;

#[async_trait::async_trait]
impl Command for Help {
    fn name(&self) -> &str {
        "help"
    }

    async fn run(&self, _args: Vec<&str>, _env: Enviroment<'async_trait>) -> Result<String, String> {
        let mut res = String::new();

        for command in COMMANDS {
            res.push_str(command.name());
            res.push_str("\n");
        }

        Ok(res)
    }
}

#[linkme::distributed_slice(COMMANDS)]
static HELP: &dyn Command = &Help;
