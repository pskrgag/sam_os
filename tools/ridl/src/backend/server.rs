use super::utils;
use crate::ast::{interface::Interface, module::Module};
use std::io::Write;
use utils::{function_to_struct, Message};

struct InterfaceCompiler<'a, W: Write> {
    interface: &'a Interface,
    buf: &'a mut W,
    messages: Vec<Message>,
}

impl<'a, W: Write> InterfaceCompiler<'a, W> {
    fn produce_enums(&mut self) {
        utils::produce_enums(self.buf, &self.messages);
    }

    fn match_payload(&self) -> String {
        let mut res = String::new();

        for (num, func) in self.interface.functions().iter().enumerate() {
            let name = func.name();
            res.push_str(
                format!(
                    "Tx::{name}(_) => self.handlers[{num}].as_mut().unwrap()(payload.clone()),\n"
                )
                .as_str(),
            );
        }

        res
    }

    fn event_loop(&mut self) {
        write!(
            self.buf,
            r#"    pub fn run(&mut self) -> Result<(), ErrorType> {{

        let mut in_msg = IpcMessage::new();
        let mut receive_buffer = [0u8; core::mem::size_of::<Tx>()];
        let mut reply_vec;

        in_msg.set_in_arena(receive_buffer.as_mut_slice());

        let mut size = self.port.receive(&mut in_msg)?;

        loop {{
            let payload: Tx = from_bytes(&in_msg.in_arena().unwrap()[..size]).unwrap();
            let reply_port = in_msg.reply_port();

            in_msg = IpcMessage::new();

            let res = match payload {{
                {}
            }};

            let reply = match res {{
                Ok(e) => RxMessage::Ok(e),
                Err(e) => RxMessage::Err(e.bits()),
            }};

            reply_vec = to_allocvec(&reply).unwrap();

            in_msg.set_in_arena(receive_buffer.as_mut_slice());
            in_msg.set_out_arena(reply_vec.as_slice());

            size = self.port.send_and_wait(reply_port, &mut in_msg)?;
        }}
    }}
"#,
            self.match_payload(),
        )
        .unwrap();
    }

    fn register_handler(&mut self) {
        write!(
            self.buf,
            r#"
    pub fn register_handler<
        M: Message,
        F: Fn(M, Arc<S>) -> Result<<M as Message>::Reply, ErrorType> + 'static,
    >(
        mut self,
        f: F,
    ) -> Self
    where
        Tx: TryInto<M>,
        Rx: From<<M as Message>::Reply>,
        <Tx as TryInto<M>>::Error: core::fmt::Debug,
    {{
        let state = self.state.clone();

        self.handlers[M::NUMBER] =
            Some(Box::new(move |message: Tx| {{
                let inner: M = message.try_into().unwrap();
                let out = f(inner, state.clone())?;
                Ok(Rx::from(out))
            }}));

        self
    }}
"#,
        )
        .unwrap();
    }

    fn make_struct(&mut self) {
        let num = self.interface.functions().len();
        let name = self.interface.name();

        writeln!(
            self.buf,
            r#"
pub struct {name}<S> {{
    port: Port,
    handlers: [Option<Box<dyn Fn(Tx) -> Result<Rx, ErrorType>>>; {num}],
    state: Arc<S>,
}}

impl<S: 'static> {name}<S> {{
    pub fn new(port: Port, s: S) -> Self {{
        Self {{ port, handlers: [None; {num}], state: Arc::new(s), }}
    }}
"#
        )
        .unwrap()
    }

    pub fn compile(mut self) {
        utils::start_mod(self.buf);
        utils::includes(self.buf);
        self.make_struct();

        for i in self.interface.functions() {
            let msg = function_to_struct(i);

            self.messages.push(msg);
        }

        self.event_loop();
        self.register_handler();

        utils::end_impl(self.buf);
        self.produce_enums();
        utils::end_mod(self.buf);
    }
}

pub fn compile_server<W: Write>(ir: Module, buf: &mut W) {
    for interface in ir.interfaces() {
        InterfaceCompiler {
            interface,
            buf,
            messages: vec![],
        }
        .compile()
    }
}
