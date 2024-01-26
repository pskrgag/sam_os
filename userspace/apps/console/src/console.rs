use crate::backend::Backend;

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

        loop {
            let read = self.b.read_byte().unwrap();
            if read == 13 {
                self.b.write_byte(b'\n');
                self.write_prompt();
            } else {
                self.b.write_byte(read);
            }
        }
    }
}
