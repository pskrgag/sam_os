use crate::ast::{
    argtype::Type,
    function::{Argument, Function},
};
use std::io::Write;

#[derive(Clone)]
pub struct Struct {
    pub data: Vec<(String, Type)>,
}

pub struct Message {
    pub tx: Struct,
    pub rx: Struct,
    pub name: String,
}

pub fn start_mod<W: Write>(buf: &mut W) {
    writeln!(buf, "#[allow(dead_code)]").unwrap();
    writeln!(buf, "#[allow(non_snake_case)]").unwrap();
    writeln!(buf, "#[allow(unused_imports)]").unwrap();
    writeln!(buf, "#[allow(unreachable_patterns)]").unwrap();
    writeln!(buf, "#[allow(private_bounds)]").unwrap();
    writeln!(buf, "mod bindings {{").unwrap();
}

pub fn end_mod<W: Write>(buf: &mut W) {
    writeln!(buf, "}}").unwrap();
}

pub fn includes<W: Write>(buf: &mut W) {
    writeln!(buf, "use rtl::handle::Handle;").unwrap();
    writeln!(buf, "use rtl::ipc::IpcMessage;").unwrap();
    writeln!(buf, "use rtl::error::ErrorType;").unwrap();
    writeln!(buf, "use serde::{{Deserialize, Serialize}};").unwrap();
    writeln!(buf, "use alloc::boxed::Box;").unwrap();
    writeln!(buf, "use postcard::{{to_allocvec, from_bytes}};").unwrap();
    writeln!(buf, "use libc::port::Port;").unwrap();
    writeln!(buf, "use alloc::sync::Arc;").unwrap();
    writeln!(buf).unwrap();
}

pub fn make_struct<W: Write>(buf: &mut W, name: &str, extra: &str, extra_contructor: &str) {
    writeln!(
        buf,
        "pub struct {} {{\n    handle: Handle, {extra}\n}}\n",
        name,
    )
    .unwrap();
    write!(buf, "impl {} {{", name).unwrap();
    writeln!(
        buf,
        r#"
    pub fn new(handle: Handle) -> Self {{
        Self {{ handle, {extra_contructor} }}
    }}
"#,
    )
    .unwrap();
}

pub fn end_impl<W: Write>(buf: &mut W) {
    writeln!(buf, "}}").unwrap();
}

fn produce_compound_enum<W: Write>(
    buf: &mut W,
    s: &Struct,
    name: &str,
    suffix: &str,
    number: usize,
) {
    writeln!(
        buf,
        "#[derive(Serialize, Deserialize, Debug, Clone)]\npub struct {name}{suffix} {{",
    )
    .unwrap();

    for data in &s.data {
        writeln!(buf, "    {}: {}", data.0, data.1).unwrap();
    }

    writeln!(buf, "}}").unwrap();

    if suffix == "Tx" {
        writeln!(
            buf,
            r#"
impl Message for {name}{suffix} {{
    const NUMBER: usize = {number};
    type Reply = {name}Rx;
}}

impl TryInto<{name}{suffix}> for Tx {{
    type Error = ();

    fn try_into(self) -> Result<{name}{suffix}, Self::Error> {{
        match self {{
            Self::{name}(e) => Ok(e),
            _ => Err(()),
        }}
    }}
}}
"#
        )
        .unwrap();
    } else {
        writeln!(
            buf,
            r#"
impl From<{name}{suffix}> for Rx {{
    fn from(value: {name}{suffix}) -> Self {{
        Self::{name}(value)
    }}
}}
"#
        )
        .unwrap();
    }
}

fn produce_final_enum<W: Write>(buf: &mut W, data: &Vec<Message>, tx: bool) {
    let name = if tx { "Tx" } else { "Rx" };

    writeln!(
        buf,
        "#[derive(Serialize, Deserialize, Debug, Clone)]\nenum {name} {{"
    )
    .unwrap();

    for msg in data {
        writeln!(buf, "    {}({}{})", msg.name, msg.name, name).unwrap();
    }

    writeln!(buf, "}}").unwrap();
}

pub fn produce_enums<W: Write>(buf: &mut W, messages: &Vec<Message>) {
    writeln!(
        buf,
        r#"
pub trait Message: Sized {{
    const NUMBER: usize;
    type Reply;
}}
"#
    )
    .unwrap();

    for (num, msg) in messages.iter().enumerate() {
        produce_compound_enum(buf, &msg.tx, &msg.name, "Tx", num);
        produce_compound_enum(buf, &msg.rx, &msg.name, "Rx", num);
    }

    produce_final_enum(buf, messages, true);
    produce_final_enum(buf, messages, false);
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
        tx: Struct { data: tx },
        rx: Struct { data: rx },
        name: f.name().to_string(),
    }
}
