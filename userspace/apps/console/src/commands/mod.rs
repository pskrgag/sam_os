use crate::cwd::Cwd;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

mod cat;
mod cd;
mod echo;
mod help;
mod ls;
mod mkdir;
mod ping;
mod touch;
mod write;

pub struct Enviroment<'a> {
    pub cwd: &'a mut Cwd,
}

#[async_trait::async_trait]
pub trait Command: Send + Sync {
    /// Command name
    fn name(&self) -> &str;

    /// Run the command
    async fn run(&self, args: Vec<&str>, env: Enviroment<'async_trait>) -> Result<String, String>;
}

#[linkme::distributed_slice]
pub(super) static COMMANDS: [&'static dyn Command] = [..];

pub fn commands() -> &'static [&'static dyn Command] {
    COMMANDS.as_ref()
}
