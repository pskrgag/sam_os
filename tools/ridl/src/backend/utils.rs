use crate::ast::{
    argtype::{BuiltinTypes, Type},
    function::{Argument, Function},
};
use std::io::Write;

#[derive(Clone)]
pub struct Struct {
    pub data: Vec<(String, Type)>,
}

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
    writeln!(buf, "use crate::alloc::borrow::ToOwned;").unwrap();
    writeln!(buf, "use serde::ser::SerializeTuple;").unwrap();
    writeln!(
        buf,
        r#"
        fn clone_into_array<T, const N: usize>(slice: &[T]) -> Result<(usize, [T; N]), ()>
    where
        T: Clone + Default + Copy
    {{
        let mut a = [T::default(); N];

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
        "#[derive(Serialize, Deserialize, Debug, Clone)]\npub struct {name} {{",
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

    wire_to_public(buf, s, name, tx);
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

fn wire_to_public<W: Write>(buf: &mut W, s: &Struct, name: &str, tx: bool) {
    let wire_name = if tx {
        wire_type_tx(name)
    } else {
        wire_type_rx(name)
    };
    let name = if tx {
        public_type_tx(name)
    } else {
        public_type_rx(name)
    };

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
            .map(|x| match &x.1 {
                Type::Sequence { inner, .. } => {
                    if **inner != Type::Builtin(BuiltinTypes::Char) {
                        format!(
                            "{name}: self.{name}.1[..self.{name}.0].to_vec()",
                            name = x.0
                        )
                    } else {
                        format!(
                            "{name}: core::str::from_utf8(&self.{name}.1[..self.{name}.0]).unwrap().to_owned()",
                            name = x.0
                        )
                    }
                }
                Type::Builtin(BuiltinTypes::Handle) => format!(
                    "{name}: Handle::new(_message.handles()[self.{name}])",
                    name = x.0
                ),
                _ => format!("{name}: self.{name}", name = x.0),
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
            .map(|x| match &x.1 {
                Type::Sequence { .. } => {
                    format!(
                        "{name}: clone_into_array(self.{name}.as_bytes()).unwrap()",
                        name = x.0
                    )
                }
                Type::Builtin(BuiltinTypes::Handle) => format!(
                    "{name}: _message.add_handle(unsafe {{ let res = self.{name}.as_raw(); core::mem::forget(self.{name}); res }})",
                    name = x.0
                ),
                _ => format!("{name}: self.{name}", name = x.0),
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
pub trait WireMessage: Sized {{
    const NUMBER: usize;
    type Reply;
}}

pub trait Message: Sized {{
    type Reply;
    type Wire;
}}

pub trait WireToPublic<T>: Sized {{
    fn try_to_public(self, _message: &IpcMessage) -> Result<T, ()>;
}}

pub trait PublicToWire<T>: Sized {{
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
        tx: Struct { data: tx },
        rx: Struct { data: rx },
        name: f.name().to_string(),
    }
}
