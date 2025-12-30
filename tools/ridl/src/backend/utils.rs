use crate::ast::{
    argtype::{BuiltinTypes, Struct, Type},
    function::{Argument, Function},
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
    writeln!(buf, "use postcard::{{to_allocvec, from_bytes, to_slice}};").unwrap();
    writeln!(buf, "use libc::port::Port;").unwrap();
    writeln!(buf, "use alloc::sync::Arc;").unwrap();
    writeln!(buf, "use alloc::string::String;").unwrap();
    writeln!(buf, "use alloc::vec::Vec;").unwrap();
    writeln!(buf, "use crate::alloc::borrow::ToOwned;").unwrap();
    writeln!(buf, "use serde::ser::SerializeTuple;").unwrap();
    writeln!(
        buf,
        r#"
        fn clone_into_array<T, const N: usize>(slice: &[T]) -> Result<(usize, [T; N]), ()>
        where
            T: Clone + Default
    {{
        let mut a: [T; N] = core::array::from_fn(|_| T::default());

        if a.as_mut().len() < slice.len() {{
            Err(())
        }} else {{
            (a[..slice.len()]).clone_from_slice(slice);
            Ok((slice.len(), a))
        }}
     }}
    "#
    )
    .unwrap();

    // Custom (de)serialization functions for slices (usize, [T; N])

    writeln!(
        buf,
        r#"
use core::mem::MaybeUninit;
use serde::de::{{Error, SeqAccess, Unexpected, Visitor}};
use serde::ser::SerializeSeq;
use serde::{{Deserializer, Serializer}};

        fn serialize<const N: usize, S, T>(t: &(usize, [T; N]), serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: Serialize,
            {{
                 {{
                      let mut seq = serializer.serialize_seq(Some(t.0 + 1))?;

                      seq.serialize_element(&t.0)?;

                      for elem in 0..t.0 {{
                          seq.serialize_element(&t.1[elem])?;
                      }}

                      seq.end()
                  }}
             }}

    fn deserialize<'de, D, const N: usize, T>(data: D) -> Result<(usize, [T; N]), D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
        {{
             {{
                  struct Vis<'de, T: Deserialize<'de>, const N: usize>(
                      (
                          core::marker::PhantomData<[T; N]>,
                          core::marker::PhantomData<&'de u8>,
                      ),
                  );

                  impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Vis<'de, T, N> {{
                      type Value = (usize, [T; N]);

                      fn expecting(&self, _formatter: &mut core::fmt::Formatter) -> core::fmt::Result {{
                          Ok(())
                      }}

                      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                      where
                          A: SeqAccess<'de>,
                      {{
                           let len = seq.next_element::<u64>()?.ok_or(Error::custom("wtf"))? as usize;
                           let mut res = [const {{ MaybeUninit::uninit() }}; N];

                           assert!(len < N);

                           for i in res.iter_mut().take(len) {{
                               i.write(seq.next_element::<T>()?.ok_or(Error::custom("wtf1"))?);
                           }}

                           Ok(unsafe {{ (len, res.map(|x| x.assume_init())) }})
                       }}
                  }}

                  data.deserialize_seq(unsafe {{ core::mem::zeroed::<Vis<T, N>>() }})
              }}
         }}
    "#
    )
    .unwrap();
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
        if data.1.is_sequence() {
            writeln!(buf, "    #[serde(serialize_with = \"serialize\")]").unwrap();
            writeln!(buf, "    #[serde(deserialize_with = \"deserialize\")]").unwrap();
        }

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
        writeln!(buf, "    pub {}: {},", data.0, data.1.as_public()).unwrap();
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

fn produce_compound_enum<W: Write>(buf: &mut W, s: &Struct, name: &str, tx: bool, number: usize) {
    produce_wire_type(buf, s, name, tx);
    produce_public_type(buf, s, name, tx);

    if tx {
        writeln!(
            buf,
            r#"
impl WireMessage for {tx} {{
    const NUMBER: usize = {number};
    type Reply = {rx};
}}

impl TryInto<{tx}> for Tx {{
    type Error = ();

    fn try_into(self) -> Result<{tx}, Self::Error> {{
        match self {{
            Self::{name}(e) => Ok(e),
            _ => Err(()),
        }}
    }}
}}
"#,
            tx = wire_type_tx(name),
            rx = wire_type_rx(name),
        )
        .unwrap();
    } else {
        writeln!(
            buf,
            r#"
impl From<{rx}> for Rx {{
    fn from(value: {rx}) -> Self {{
        Self::{name}(value)
    }}
}}

impl TryInto<{rx}> for Rx {{
    type Error = ();

    fn try_into(self) -> Result<{rx}, Self::Error> {{
        match self {{
            Self::{name}(e) => Ok(e),
            _ => Err(()),
        }}
    }}
}}
"#,
            rx = wire_type_rx(name),
        )
        .unwrap();
    }

    wire_to_public(buf, s);
}

fn produce_final_enum<W: Write>(buf: &mut W, data: &Vec<Message>, tx: bool) {
    let wire_suffix = if tx { "TxWire" } else { "RxWire" };
    let name = if tx { "Tx" } else { "Rx" };

    writeln!(
        buf,
        "#[derive(Serialize, Deserialize, Debug, Clone)]\nenum {name} {{"
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
            if **inner == Type::Builtin(BuiltinTypes::Char) {
                format!("core::str::from_utf8(&{name}.1[..{name}.0]).unwrap().to_owned()",)
            } else if matches!(**inner, Type::Builtin(_)) {
                format!("{name}.1[..{name}.0].to_vec()",)
            } else {
                format!("(&{name}.1[..{name}.0]).into_iter().map(|x| x.clone().try_to_public(_message).unwrap()).collect()")
            }
        }
        Type::Builtin(BuiltinTypes::Handle) => format!("Handle::new(_message.handles()[{name}])"),
        Type::Struct(_) => format!("{name}.try_to_public().unwrap()"),
        _ => name.to_string(),
    }
}

fn type_public_to_wire<S: AsRef<str>>(tp: &Type, var: S) -> String {
    let name = var.as_ref();

    match tp {
        Type::Sequence { inner, .. } => {
            let f = match **inner {
                Type::Builtin(BuiltinTypes::Char) => ".as_bytes()",
                Type::Struct(_) => ".iter().map(|x| x.clone().try_to_wire(_message).unwrap()).collect::<Vec<_>>().as_slice()",
                _ => ".as_slice()",
            };

            format!(
                "clone_into_array({name}{f}).unwrap()",
            )
        }
        Type::Builtin(BuiltinTypes::Handle) => format!(
            "_message.add_handle(unsafe {{ let res = {name}.as_raw(); core::mem::forget({name}); res }})",
        ),
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
    fn try_to_public(self, _message: &IpcMessage) -> Result<{name}, ()> {{
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
    fn try_to_wire(self, _message: &mut IpcMessage) -> Result<{wire_name}, ()> {{
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

pub fn produce_enums<W: Write>(buf: &mut W, messages: &Vec<Message>) {
    writeln!(
        buf,
        r#"
trait WireMessage: Sized {{
    const NUMBER: usize;
    type Reply;
}}

trait Message: Sized {{
    type Reply;
    type Wire;
}}

trait WireToPublic<T>: Sized {{
    fn try_to_public(self, _message: &IpcMessage) -> Result<T, ()>;
}}

trait PublicToWire<T>: Sized {{
    fn try_to_wire(self, _message: &mut IpcMessage) -> Result<T, ()>;
}}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum RxMessage {{
    Ok(Rx),
    Err(usize),
}}
"#
    )
    .unwrap();

    for (num, msg) in messages.iter().enumerate() {
        produce_compound_enum(buf, &msg.tx, &msg.name, true, num);
        produce_compound_enum(buf, &msg.rx, &msg.name, false, num);
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
            .map(|x| format!("pub {}: {},", x.0, x.1.as_public()))
            .collect::<Vec<_>>()
            .join("\n"),
        name = s.name,
    )
    .unwrap();

    wire_to_public(buf, s);
}
