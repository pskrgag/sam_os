use crate::ast::{
    argtype::{BuiltinTypes, Struct, Type},
    function::{Argument, Function},
    interface::Interface,
};
use std::io::Write;

static WIRE_SUFFIX: &str = "Wire";

pub fn wire_type_tx(name: &str) -> String {
    format!("{name}Tx{WIRE_SUFFIX}")
}

pub fn public_type_tx(name: &str) -> String {
    format!("{name}Tx")
}

pub fn wire_type_rx(name: &str) -> String {
    format!("{name}Rx{WIRE_SUFFIX}")
}

pub fn public_type_rx(name: &str) -> String {
    format!("{name}Rx")
}

pub struct Message {
    pub tx: Struct,
    pub rx: Struct,
    pub name: String,
}

pub fn start_mod<W: Write>(buf: &mut W, suffix: &str) {
    writeln!(buf, "#[allow(dead_code)]").unwrap();
    writeln!(buf, "#[allow(non_snake_case)]").unwrap();
    writeln!(buf, "#[allow(unused_imports)]").unwrap();
    writeln!(buf, "#[allow(unreachable_patterns)]").unwrap();
    writeln!(buf, "#[allow(private_bounds)]").unwrap();
    writeln!(buf, "#[allow(clippy::type_complexity)]").unwrap();
    writeln!(buf, "#[allow(clippy::missing_transmute_annotations)]").unwrap();
    writeln!(buf, "#[allow(clippy::large_enum_variant)]").unwrap();
    writeln!(buf, "#[allow(forgetting_references)]").unwrap();
    writeln!(buf, "mod bindings_{suffix} {{").unwrap();
}

pub fn end_mod<W: Write>(buf: &mut W) {
    writeln!(buf, "}}").unwrap();
}

pub fn includes<W: Write>(buf: &mut W) {
    writeln!(buf, "use libc::handle::Handle;").unwrap();
    writeln!(buf, "use rtl::ipc::IpcMessage;").unwrap();
    writeln!(buf, "use rtl::error::ErrorType;").unwrap();
    writeln!(buf, "use serde::{{Deserialize, Serialize}};").unwrap();
    writeln!(buf, "use alloc::boxed::Box;").unwrap();
    writeln!(
        buf,
        "use postcard::{{to_vec, to_allocvec, from_bytes, to_slice}};"
    )
    .unwrap();
    writeln!(buf, "use rokio::port::Port;").unwrap();
    writeln!(buf, "use alloc::sync::Arc;").unwrap();
    writeln!(buf, "use heapless::String as HLString;").unwrap();
    writeln!(buf, "use heapless::Vec as HLVec;").unwrap();
    writeln!(buf, "use alloc::vec::Vec;").unwrap();
    writeln!(buf, "use crate::alloc::borrow::ToOwned;").unwrap();
    writeln!(buf, "use serde::ser::SerializeTuple;").unwrap();
    writeln!(buf, "use rtl::handle::Handle as RawHandle;").unwrap();
    writeln!(buf).unwrap();
}

pub fn end_impl<W: Write>(buf: &mut W) {
    writeln!(buf, "}}").unwrap();
}

fn produce_wire_type<W: Write>(buf: &mut W, s: &Struct, name: &str, tx: bool) {
    let name = if tx {
        wire_type_tx(name)
    } else {
        wire_type_rx(name)
    };

    writeln!(
        buf,
        "#[derive(Serialize, Deserialize, Debug, Clone)]\nstruct {name} {{",
    )
    .unwrap();

    for data in &s.data {
        writeln!(buf, "    pub {}: {},", data.0, data.1.as_wire()).unwrap();
    }

    writeln!(buf, "}}").unwrap();
}

fn produce_public_type<W: Write>(buf: &mut W, s: &Struct, name: &str, tx: bool) {
    let struct_name = if tx {
        public_type_tx(name)
    } else {
        public_type_rx(name)
    };

    writeln!(buf, "#[derive(Debug)]\npub struct {struct_name} {{",).unwrap();

    for data in &s.data {
        writeln!(buf, "    pub {}: {},", data.0, data.1.as_rust()).unwrap();
    }

    writeln!(buf, "}}").unwrap();

    if tx {
        writeln!(
            buf,
            r#"
impl Message for {struct_name} {{
    type Reply = {};
    type Wire = {};
}}
"#,
            public_type_rx(name),
            wire_type_tx(name),
        )
        .unwrap();
    } else {
        writeln!(
            buf,
            r#"
impl Message for {struct_name} {{
    type Reply = ();
    type Wire = {};
}}
"#,
            wire_type_rx(name),
        )
        .unwrap();
    }
}

fn produce_compound_enum<W: Write>(
    buf: &mut W,
    s: &Struct,
    func_name: &str,
    iface_name: &str,
    tx: bool,
) {
    produce_wire_type(buf, s, func_name, tx);
    produce_public_type(buf, s, func_name, tx);

    if tx {
        writeln!(
            buf,
            r#"
impl WireMessage for {tx} {{
    type Reply = {rx};
}}

impl TryInto<{tx}> for Tx{iface_name} {{
    type Error = ();

    fn try_into(self) -> Result<{tx}, Self::Error> {{
        match self {{
            Self::{func_name}(e) => Ok(e),
            _ => Err(()),
        }}
    }}
}}
"#,
            tx = wire_type_tx(func_name),
            rx = wire_type_rx(func_name),
        )
        .unwrap();
    } else {
        writeln!(
            buf,
            r#"
impl From<{rx}> for Rx{iface_name} {{
    fn from(value: {rx}) -> Self {{
        Self::{func_name}(value)
    }}
}}

impl TryInto<{rx}> for Rx{iface_name} {{
    type Error = ();

    fn try_into(self) -> Result<{rx}, Self::Error> {{
        match self {{
            Self::{func_name}(e) => Ok(e),
            _ => Err(()),
        }}
    }}
}}
"#,
            rx = wire_type_rx(func_name),
        )
        .unwrap();
    }

    wire_to_public(buf, s);
}

fn produce_final_enum<W: Write>(buf: &mut W, data: &Vec<Message>, iface_name: &str, tx: bool) {
    let wire_suffix = if tx { "TxWire" } else { "RxWire" };
    let name = if tx { "Tx" } else { "Rx" };

    writeln!(
        buf,
        "#[derive(Serialize, Deserialize, Debug, Clone)]\nenum {name}{iface_name} {{"
    )
    .unwrap();

    for msg in data {
        writeln!(buf, "    {}({}{}),", msg.name, msg.name, wire_suffix).unwrap();
    }

    writeln!(buf, "}}").unwrap();
}

fn type_wire_to_public<S: AsRef<str>>(tp: &Type, var: S) -> String {
    let name = var.as_ref();

    match tp {
        Type::Sequence { inner, .. } => {
            if matches!(**inner, Type::Builtin(_)) {
                format!("{name}.clone()")
            } else {
                format!(
                    "{name}.into_iter().map(|x| x.clone().try_to_public(_message).unwrap()).collect()"
                )
            }
        }
        Type::Builtin(BuiltinTypes::Handle) => format!("Handle::new(_message.handles()[{name}])"),
        Type::Struct(_) => format!("{name}.try_to_public(_message).unwrap()"),
        _ => name.to_string(),
    }
}

fn type_public_to_wire<S: AsRef<str>>(tp: &Type, var: S) -> String {
    let name = var.as_ref();

    match tp {
        Type::Sequence { inner, count } => {
            let f = match **inner {
                Type::Builtin(_) => ".clone()",
                Type::Struct(_) => &format!(
                    ".iter().map(|x| x.clone().try_to_wire(_message).unwrap()).collect::<HLVec<_, {count}>>()"
                ),
                _ => todo!(),
            };

            format!("{name}{f}.clone()",)
        }
        Type::Builtin(BuiltinTypes::Handle) => format!(
            "_message.add_handle(unsafe {{ let res = {name}.as_raw(); core::mem::forget({name}); res }})",
        ),
        Type::Struct(_) => format!("{name}.try_to_wire(_message).unwrap()"),
        _ => name.to_string(),
    }
}

fn wire_to_public<W: Write>(buf: &mut W, s: &Struct) {
    let name = &s.name;
    let wire_name = format!("{}Wire", s.name);

    writeln!(
        buf,
        r#"
impl WireToPublic<{name}> for {wire_name} {{
    fn try_to_public(self, _message: &IpcMessage) -> Result<{name}, ErrorType> {{
        Ok({name} {{
            {}
        }})
    }}
}}
"#,
        s.data
            .iter()
            .map(|x| {
                let expr = type_wire_to_public(&x.1, format!("self.{name}", name = x.0));
                format!("{name}: {expr}", name = x.0)
            })
            .collect::<Vec<_>>()
            .join(", "),
    )
    .unwrap();

    writeln!(
        buf,
        r#"
impl PublicToWire<{wire_name}> for {name} {{
    fn try_to_wire(self, _message: &mut IpcMessage) -> Result<{wire_name}, ErrorType> {{
        Ok({wire_name} {{
            {}
        }})
    }}
}}
"#,
        s.data
            .iter()
            .map(|x| {
                let expr = type_public_to_wire(&x.1, format!("self.{name}", name = x.0));
                format!("{name}: {expr}", name = x.0)
            })
            .collect::<Vec<_>>()
            .join(", "),
    )
    .unwrap();
}

fn produce_send_struct<W: Write>(buf: &mut W, interface: &Interface, message: &Message) {
    let int_name = interface.name();

    writeln!(
        buf,
        r#"
    pub struct {int_name}{message_name}Reply {{
        port: RawHandle,
        reply_port: RawHandle,
    }}

    impl {int_name}{message_name}Reply {{
        pub fn reply(self {args}) -> Result<(), ErrorType> {{
            let mut out_msg = IpcMessage::new();
            let _message = &mut out_msg;
            let msg = {wire_name_rx} {{ {} }};
            let wire = RxMessage{iface_name}::Ok(Rx{iface_name}::{message_name}(msg));
            let vec = to_allocvec(&wire).unwrap();
            let port = core::mem::ManuallyDrop::new(unsafe {{ Port::new(Handle::new(self.port)) }});

            _message.set_out_arena(vec.as_slice());

            port.reply(libc::handle::Handle::new(self.reply_port), &out_msg )
        }}
    }}
"#,
        message
            .rx
            .data
            .iter()
            .map(|x| {
                let expr = type_public_to_wire(&x.1, &x.0);
                format!("{name}: {expr}", name = x.0)
            })
            .collect::<Vec<_>>()
            .join(", "),
        wire_name_rx = wire_type_rx(&message.name),
        message_name = message.name,
        args = message
            .rx
            .data
            .iter()
            .map(|x| format!(", {}: {}", x.0, x.1.as_arg()))
            .collect::<Vec<_>>()
            .join(" "),
        iface_name = interface.name(),
    )
    .unwrap();
}

pub fn produce_server_public_enum<W: Write>(
    buf: &mut W,
    interface: &Interface,
    messages: &Vec<Message>,
) {
    let int_name = interface.name();

    for msg in messages {
        produce_send_struct(buf, interface, msg);
    }

    writeln!(buf, "pub enum {int_name}Request {{").unwrap();
    for msg in messages {
        let message_name = &msg.name;

        writeln!(buf, "    {message_name}{{ value: {message_name}Tx, responder: {int_name}{message_name}Reply }},").unwrap();
    }
    writeln!(buf, "}}").unwrap();

    writeln!(
        buf,
        r#"
impl Tx{iface_name} {{
    fn to_public(
        self,
        old_message: &IpcMessage,
        port: &Port)
    -> Result<{int_name}Request, ErrorType> 
    {{
        match self {{
            {}
        }}
    }}
}}
"#,
        messages
            .iter()
            .map(|x| {
                let message_name = &x.name;

                format!(r#"
                    Self::{strname}(x) => {{
                        x.try_to_public(old_message).map(|value| {{
                            {int_name}Request::{strname} {{ value, responder: {int_name}{message_name}Reply {{
                                    port: unsafe {{ port.handle().as_raw() }},
                                    reply_port: old_message.reply_port(),
                                }}
                            }}
                        }})
                    }}"#, 
                    strname = x.name)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        iface_name = interface.name(),
    )
    .unwrap();
}

pub fn common_traits<W: Write>(buf: &mut W) {
    writeln!(
        buf,
        r#"
trait WireMessage: Sized {{
    type Reply;
}}

trait Message: Sized {{
    type Reply;
    type Wire;
}}

trait WireToPublic<T>: Sized {{
    fn try_to_public(self, _message: &IpcMessage) -> Result<T, ErrorType>;
}}

trait PublicToWire<T>: Sized {{
    fn try_to_wire(self, _message: &mut IpcMessage) -> Result<T, ErrorType>;
}}
"#
    )
    .unwrap();
}

pub fn produce_enums<W: Write>(buf: &mut W, messages: &Vec<Message>, name: &str) {
    writeln!(
        buf,
        r#"
#[derive(Serialize, Deserialize, Debug, Clone)]
enum RxMessage{name} {{
    Ok(Rx{name}),
    Err(usize),
}}
"#
    )
    .unwrap();

    for msg in messages.iter() {
        produce_compound_enum(buf, &msg.tx, &msg.name, name, true);
        produce_compound_enum(buf, &msg.rx, &msg.name, name, false);
    }

    produce_final_enum(buf, messages, name, true);
    produce_final_enum(buf, messages, name, false);
}

pub fn function_to_struct(f: &Function) -> Message {
    let mut rx = vec![];
    let mut tx = vec![];

    for arg in f.args() {
        match arg {
            Argument::In(tp, name) => {
                tx.push((name.clone(), tp.clone()));
            }
            Argument::Out(tp, name) => rx.push((name.clone(), tp.clone())),
        }
    }

    Message {
        tx: Struct {
            data: tx,
            name: format!("{}Tx", f.name()),
        },
        rx: Struct {
            data: rx,
            name: format!("{}Rx", f.name()),
        },
        name: f.name().to_string(),
    }
}

pub fn produce_struct<W: Write>(buf: &mut W, s: &Struct) {
    writeln!(
        buf,
        r#"
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct {name}Wire {{
    {}
}}

#[derive(Debug, Default, Clone)]
pub struct {name} {{
    {}
}}
"#,
        s.data
            .iter()
            .map(|x| format!("pub {}: {},", x.0, x.1.as_wire()))
            .collect::<Vec<_>>()
            .join("\n"),
        s.data
            .iter()
            .map(|x| format!("pub {}: {},", x.0, x.1.as_rust()))
            .collect::<Vec<_>>()
            .join("\n"),
        name = s.name,
    )
    .unwrap();

    wire_to_public(buf, s);
}
