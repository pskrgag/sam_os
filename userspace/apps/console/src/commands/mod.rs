use alloc::boxed::Box;
use alloc::string::String;

mod echo;
mod help;

lazy_static::lazy_static! {
    pub static ref COMMANDS: [Box<dyn Command + Sync + Send>; 2] = [Box::new(help::Help{}), Box::new(echo::Echo{})];
}

pub trait Command {
    fn name(&self) -> &str;
    fn exe(&self, args: &[&str]) -> Option<String>;
    fn help(&self) -> &str;
}
