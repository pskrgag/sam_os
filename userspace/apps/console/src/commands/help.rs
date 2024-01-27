use super::Command;
use alloc::string::String;

pub struct Help{}

impl Command for Help {
    fn name(&self) -> &str {
        "help"
    }

    fn exe(&self, args: &[&str]) -> Option<String> {
        Some(String::from("I am a stupid console, I cannot do anything for now"))
    }

    fn help(&self) -> &str {
        ""
    }
}

unsafe impl Sync for Help {}
unsafe impl Send for Help {}
