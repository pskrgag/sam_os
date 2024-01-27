use super::*;
use alloc::string::String;

pub struct Help {}

impl Command for Help {
    fn name(&self) -> &str {
        "help"
    }

    fn exe(&self, args: &[&str]) -> Option<String> {
        let mut s = String::from("Supported commands:\n");

        COMMANDS.iter().for_each(|x| {
            s.push_str(x.name());
            s.push_str(": ");
            s.push_str(x.help());
            s.push_str("\n");
        });

        Some(s)
    }

    fn help(&self) -> &str {
        "Shows this message"
    }
}

unsafe impl Sync for Help {}
unsafe impl Send for Help {}
