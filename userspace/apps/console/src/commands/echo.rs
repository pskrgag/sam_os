use super::{Command, COMMANDS, Enviroment};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;

struct Echo;

#[async_trait::async_trait]
impl Command for Echo {
    fn name(&self) -> &str {
        "echo"
    }

    async fn run(&self, args: Vec<&str>, _env: Enviroment<'async_trait>) -> Result<String, String> {
        Ok(args.join(" "))
    }
}

#[linkme::distributed_slice(COMMANDS)]
static ECHO: &dyn Command = &Echo;
