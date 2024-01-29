use crate::backend::Backend;
use crate::commands::COMMANDS;
use alloc::format;
use alloc::vec::Vec;

pub struct Console<B: Backend + Default> {
    b: B,
}

impl<B: Backend + Default> Console<B> {
    pub fn new() -> Self {
        Self { b: B::default() }
    }

    fn write_prompt(&self) {
        // write prompt
        self.b.write_bytes(b"> ");
    }

    pub fn exec(&self) -> ! {
        self.write_prompt();

        let mut num_command_name = [0u8; 1000];
        let mut index = 0;

        loop {
            let read = self.b.read_byte().unwrap();

            match read {
                // Enter
                0xd => {
                    self.b.write_byte(b'\n');

                    if index != 0 {
                        if let Ok(name) = core::str::from_utf8(&num_command_name[..index]) {
                            let words = name.split_whitespace().collect::<Vec<_>>();
                            let args = &words[1..];

                            let cmd = COMMANDS.iter().find(|x| x.name() == words[0]);

                            if let Some(cmd) = cmd {
                                if let Some(out) = cmd.exe(args) {
                                    self.b.write_bytes(format!("{}\n", out).as_bytes());
                                }
                            } else {
                                self.b.write_bytes(
                                    format!("Unknown command '{}'\n", name).as_bytes(),
                                );
                            }
                        }
                    }

                    index = 0;
                    self.write_prompt();
                }
                // Backspace
                0x7f | 0x8 => {
                    if index != 0 {
                        self.b.write_bytes(&[0x8, b' ', 0x8]);
                        index -= 1;
                    }
                }
                // Anything else
                _ => {
                    if index >= core::mem::size_of_val(&num_command_name) {
                        index = 0;
                        self.b.write_bytes(b"Too long name");
                    }

                    num_command_name[index] = read;
                    index += 1;
                    self.b.write_byte(read);
                }
            }
        }
    }
}
