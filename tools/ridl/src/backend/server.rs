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
                    "Tx::{name}(_) => self.handlers[{num}].as_mut().unwrap()(payload.clone(), &in_msg, &mut out_msg),\n"
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
            let mut out_msg = IpcMessage::new();

            let res = match payload {{
                {}
            }};

            in_msg = out_msg;

            let reply = match res {{
                Ok(e) => RxMessage::Ok(e),
                Err(e) => RxMessage::Err(e.bits()),
            }};

            reply_vec = to_allocvec(&reply).unwrap();

            in_msg.set_in_arena(receive_buffer.as_mut_slice());
            in_msg.set_out_arena(reply_vec.as_slice());

            size = self.port.send_and_wait(libc::handle::Handle::new(reply_port), &mut in_msg)?;
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
        Tx: TryInto<<M as Message>::Wire>,
        <M as Message>::Wire: WireMessage,
        <M as Message>::Wire: WireToPublic<M>,
        <M as Message>::Reply: Message,
        <M as Message>::Reply: PublicToWire<<<M as Message>::Reply as Message>::Wire>,
        <Tx as TryInto<<M as Message>::Wire>>::Error: core::fmt::Debug,
        Rx: From<<<M as Message>::Reply as Message>::Wire>,
        // <M as Message>::Reply: Message,
        // <<M as Message>::Reply as Message>::Wire: TryInto<Rx>,
    {{
        let state = self.state.clone();

        self.handlers[<M as Message>::Wire::NUMBER] =
            Some(Box::new(move |message: Tx, in_msg: &IpcMessage, out_msg: &mut IpcMessage| {{
                let wire: <M as Message>::Wire = message.try_into().unwrap();
                let public = wire.try_to_public(in_msg).unwrap();
                let out = f(public, state.clone())?;
                let wire: <<M as Message>::Reply as Message>::Wire = out.try_to_wire(out_msg).unwrap();

                Ok(Rx::from(<<<M as Message>::Reply as Message>::Wire as TryInto<_>>::try_into(wire).unwrap()))
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
    handlers: [Option<Box<dyn Fn(Tx, &IpcMessage, &mut IpcMessage) -> Result<Rx, ErrorType>>>; {num}],
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
