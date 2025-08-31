use super::utils::{function_to_struct, Message};
use crate::{
    ast::{
        argtype::{BuiltinTypes, Type},
        function::Function,
        interface::Interface,
        module::Module,
    },
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
            write!(self.buf, ", {}: {}", arg.0, arg.1.as_arg()).unwrap();
        }

        {
            writeln!(
                self.buf,
                r#") -> Result<{name}Rx, ErrorType> {{
        let mut message = IpcMessage::new();
        let data = Tx::{name}({wire_name_tx} {{ {} }});
        let data_vec = to_allocvec(&data).unwrap();
        let mut receive_buffer = [0u8; core::mem::size_of::<{wire_name_tx}>()];

        message.set_out_arena(data_vec.as_slice());
        message.set_in_arena(receive_buffer.as_mut_slice());

        self.port.call(&mut message)?;

        let res: RxMessage = from_bytes(message.in_data.unwrap()).unwrap();

        let wire: {name}RxWire = match res {{
            RxMessage::Ok(e) => Ok::<{name}RxWire, ErrorType>(e.try_into().unwrap()),
            RxMessage::Err(e) => Err(unsafe {{ core::mem::transmute::<_, ErrorType>(e) }}),
        }}?;

        Ok(wire.try_to_public(&message).unwrap())
"#,
                msg.tx
                    .data
                    .iter()
                    .map(|x| match &x.1 {
                        Type::Sequence { .. } => {
                            format!(
                                "{name}: clone_into_array({name}.as_bytes()).unwrap()",
                                name = x.0
                            )
                        }
                        Type::Builtin(BuiltinTypes::Handle) => format!(
                            "{name}: message.add_handle(unsafe {{ {name}.as_raw() }})",
                            name = x.0
                        ),
                        _ => x.0.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                name = f.name(),
                wire_name_tx = utils::wire_type_tx(f.name()),
            )
            .unwrap();

            writeln!(self.buf, "    }}").unwrap();
        }

        self.messages.push(msg);
    }

    fn produce_enums(&mut self) {
        utils::produce_enums(self.buf, &self.messages);
    }

    fn make_struct(&mut self) {
        let name = self.interface.name();

        writeln!(
            self.buf,
            r#"
pub struct {name} {{
    port: Port,
}}

impl {name} {{
    pub fn new(port: Port) -> Self {{
        Self {{ port }}
    }}
"#
        )
        .unwrap()
    }

    pub fn compile(mut self) {
        utils::start_mod(self.buf);
        utils::includes(self.buf);
        self.make_struct();

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
