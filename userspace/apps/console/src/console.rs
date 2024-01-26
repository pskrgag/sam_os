use crate::backend::Backend;
use crate::commands::COMMANDS;
use alloc::format;

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

            if read == 13 {
                self.b.write_byte(b'\n');

                if index != 0 {
                    if let Ok(name) = core::str::from_utf8(&num_command_name[..index]) {
                        for i in COMMANDS.iter() {
                            if i.name() == name {
                                if let Some(out) = i.exe(&[]) {
                                    self.b.write_bytes(format!("{}\n", out).as_bytes());
                                }
                            }
                        }
                    }
                }

                index = 0;
                self.write_prompt();
            } else {
                if index >= 1000 {
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
