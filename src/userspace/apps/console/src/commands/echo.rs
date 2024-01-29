use super::Command;
use alloc::string::String;

pub struct Echo {}

impl Command for Echo {
    fn name(&self) -> &str {
        "echo"
    }

    fn exe(&self, args: &[&str]) -> Option<String> {
        let mut s = String::new();
        args.iter().for_each(|x| {
            s.push_str(x);
            s.push_str(" ");
        });
        Some(s)
    }

    fn help(&self) -> &str {
        "Echoes arguments on the console"
    }
}

unsafe impl Sync for Echo {}
unsafe impl Send for Echo {}
