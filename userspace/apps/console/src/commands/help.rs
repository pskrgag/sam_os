use super::{Command, Enviroment, COMMANDS};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

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
