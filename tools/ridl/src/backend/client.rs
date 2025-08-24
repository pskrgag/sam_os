use super::utils::{function_to_struct, Message};
use crate::{
    ast::{function::Function, interface::Interface, module::Module},
    backend::utils,
};
use std::io::Write;

struct InterfaceCompiler<'a, W: Write> {
    interface: &'a Interface,
    buf: &'a mut W,
    messages: Vec<Message>,
}

impl<'a, W: Write> InterfaceCompiler<'a, W> {
    fn compile_function(&mut self, f: &Function) {
        let msg = function_to_struct(f);

        write!(self.buf, "    pub fn {}(&self", f.name()).unwrap();

        for arg in &msg.tx.data {
            write!(self.buf, ", {}: {}", arg.0, arg.1).unwrap();
        }

        {
            writeln!(
                self.buf,
                r#") -> Result<{}Rx, ErrorType> {{
        let mut message = IpcMessage::new();
        let data = Tx::{}({}Tx {{ {} }});
        let data_vec = to_allocvec(&data).unwrap();
        let mut receive_buffer = [0u8; core::mem::size_of::<{}Tx>()];

        message.set_out_arena(data_vec.as_slice());
        message.set_in_arena(receive_buffer.as_mut_slice());

        Port::new(self.handle).call(&mut message)?;

        let res = from_bytes(message.in_data.unwrap()).unwrap();
        Ok(res)"#,
                f.name(),
                f.name(),
                f.name(),
                msg.tx
                    .data
                    .iter()
                    .map(|x| x.0.clone())
                    .collect::<Vec<_>>()
                    .join(","),
                f.name(),
            )
            .unwrap();

            writeln!(self.buf, "    }}").unwrap();
        }

        self.messages.push(msg);
    }

    fn produce_enums(&mut self) {
        utils::produce_enums(self.buf, &self.messages);
    }

    pub fn compile(mut self) {
        utils::start_mod(self.buf);
        utils::includes(self.buf);
        utils::make_struct(self.buf, self.interface.name(), "", "");

        for func in self.interface.functions() {
            self.compile_function(func);
        }
        utils::end_impl(self.buf);

        self.produce_enums();
        utils::end_mod(self.buf);
    }
}

pub fn compile_client<W: Write>(ir: Module, buf: &mut W) {
    for interface in ir.interfaces() {
        InterfaceCompiler {
            interface,
            buf,
            messages: vec![],
        }
        .compile()
    }
}
