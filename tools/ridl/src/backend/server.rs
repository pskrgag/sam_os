use super::utils;
use crate::ast::{argtype::Struct, interface::Interface, module::Module};
use std::io::Write;
use utils::{function_to_struct, Message};

struct InterfaceCompiler<'a, W: Write> {
    interface: &'a Interface,
    buf: &'a mut W,
    messages: Vec<Message>,
    structs: &'a Vec<Struct>,
}

impl<'a, W: Write> InterfaceCompiler<'a, W> {
    fn produce_enums(&mut self) {
        utils::produce_enums(self.buf, &self.messages);
        utils::produce_server_public_enum(self.buf, self.interface, &self.messages);
    }

    fn event_loop(&mut self) {
        let name = self.interface.name();

        write!(
            self.buf,
            r#"
    impl<{traits}> {name}<F, Fut> 
    {{
        async fn run(&mut self) -> Result<(), ErrorType> {{
            loop {{
                let mut receive_buffer: Vec<u8> = Vec::new();
                let mut in_msg = IpcMessage::new();

                receive_buffer.resize(core::mem::size_of::<Tx>() + 1, 0);

                in_msg.set_in_arena(receive_buffer.as_mut_slice());
                let size = self.port.receive(&mut in_msg).await?;

                let payload: Tx = from_bytes(&in_msg.in_arena().unwrap()[..size]).unwrap();
                let public = payload.to_public(&in_msg, &mut self.port)?;

                match (self.handler)(public).await {{
                    Ok(_) => {{}}, // message has been sent by closure
                    Err(e) => {{
                        let res = RxMessage::Err(e.into());
                        let res = to_allocvec(&res).unwrap();
                        let mut msg = IpcMessage::new();

                        msg.set_out_arena(res.as_slice());
                        self.port.reply(Handle::new(in_msg.reply_port()), &mut msg)?;
                    }},
                }}
            }}
        }}
    }}
"#,
            traits = self.traits()
        )
        .unwrap();
    }

    fn traits(&self) -> String {
        let name = self.interface.name();
        format!("F: FnMut({name}Request) -> Fut,\nFut: Future<Output = Result<(), ErrorType>> + 'static")
    }

    fn register_handler(&mut self) {
        let name = self.interface.name();

        write!(
            self.buf,
            r#"
    impl<{traits}> {name}<F, Fut>
    {{
        pub async fn for_each(port: Port, f: F) -> Result<(), ErrorType> {{
            let mut new = Self {{ handler: Box::new(f), port }};
            new.run().await
        }}
    }}
"#,
            traits = self.traits()
        )
        .unwrap();
    }

    fn make_struct(&mut self) {
        let name = self.interface.name();

        writeln!(
            self.buf,
            r#"
pub struct {name}<{traits}>{{
    port: Port,
    handler: Box<F>
}}

unsafe impl<{traits}> Send for {name}<F, Fut> {{ }}
"#,
            traits = self.traits()
        )
        .unwrap()
    }

    pub fn compile(mut self) {
        utils::start_mod(self.buf, self.interface.name());
        utils::includes(self.buf);

        for s in self.structs {
            utils::produce_struct(self.buf, s);
        }

        self.make_struct();
        for i in self.interface.functions() {
            let msg = function_to_struct(i);

            self.messages.push(msg);
        }

        self.event_loop();
        // self.handle_one();

        self.register_handler();
        self.produce_enums();

        utils::end_mod(self.buf);
    }
}

pub fn compile_server<W: Write>(ir: Module, buf: &mut W, _dispatch_loop: bool) {
    for interface in ir.interfaces() {
        InterfaceCompiler {
            interface,
            buf,
            messages: vec![],
            structs: ir.structs(),
        }
        .compile()
    }
}
