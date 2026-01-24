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

        write!(self.buf, "    pub async fn {}(&self", f.name()).unwrap();

        for arg in &msg.tx.data {
            write!(self.buf, ", {}: {}", arg.0, arg.1.as_arg()).unwrap();
        }

        {
            writeln!(
                self.buf,
                r#") -> Result<{name}Rx, ErrorType> {{
        let mut _message = IpcMessage::new();
        let data = Tx{iface_name}::{name}({wire_name_tx} {{ {} }});
        let data_vec = to_allocvec(&data).unwrap();
        let mut receive_buffer = [0u8; core::mem::size_of::<RxMessage{iface_name}>()];

        _message.set_out_arena(data_vec.as_slice());
        _message.set_in_arena(receive_buffer.as_mut_slice());

        let size = self.port.call(&mut _message).await?;
        let res: RxMessage{iface_name} = from_bytes(&_message.in_data.unwrap()[..size]).unwrap();

        let wire: {name}RxWire = match res {{
            RxMessage{iface_name}::Ok(e) => Ok::<{name}RxWire, ErrorType>(e.try_into().unwrap()),
            RxMessage{iface_name}::Err(e) => Err(unsafe {{ core::mem::transmute::<_, ErrorType>(e) }}),
        }}?;

        Ok(wire.try_to_public(&_message).unwrap())
"#,
                msg.tx
                    .data
                    .iter()
                    .map(|x| match &x.1 {
                        Type::Sequence { .. } => {
                            format!("{name}: {name}.clone()", name = x.0)
                        }
                        Type::Builtin(BuiltinTypes::Handle) => format!(
                            "{name}: _message.add_handle(unsafe {{ {name}.as_raw() }})",
                            name = x.0
                        ),
                        _ => x.0.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                name = f.name(),
                wire_name_tx = utils::wire_type_tx(f.name()),
                iface_name = self.interface.name(),
            )
            .unwrap();

            writeln!(self.buf, "    }}").unwrap();
        }

        self.messages.push(msg);
    }

    fn produce_enums(&mut self) {
        utils::produce_enums(self.buf, &self.messages, self.interface.name());
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
        self.make_struct();

        for func in self.interface.functions() {
            self.compile_function(func);
        }
        utils::end_impl(self.buf);

        self.produce_enums();
    }
}

pub fn compile_client<W: Write>(ir: Module, buf: &mut W) {
    utils::start_mod(buf, ir.name());
    utils::includes(buf);
    utils::common_traits(buf);
    for s in ir.structs() {
        utils::produce_struct(buf, s);
    }

    for interface in ir.interfaces() {
        InterfaceCompiler {
            interface,
            buf,
            messages: vec![],
        }
        .compile()
    }

    utils::end_mod(buf);
}
