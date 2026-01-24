use std::collections::HashMap;
use strum_macros::Display;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Struct {
    pub data: Vec<(String, Type)>,
    pub name: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enum {
    pub name: String,
    pub inner: Box<Type>,
    pub entries: Vec<String>,
}

#[derive(Clone, Copy, Debug, Display, Hash, PartialEq, Eq)]
pub enum BuiltinTypes {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Char,
    Handle,
    USize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Builtin(BuiltinTypes),
    Sequence { inner: Box<Type>, count: usize },
    Struct(Struct),
    Enum(Enum),
}

lazy_static::lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, BuiltinTypes> =
        HashMap::from([
            ("I8", BuiltinTypes::I8),
            ("U8", BuiltinTypes::U8),
            ("I16", BuiltinTypes::I16),
            ("U16", BuiltinTypes::U16),
            ("I32", BuiltinTypes::I32),
            ("U32", BuiltinTypes::U32),
            ("I64", BuiltinTypes::I64),
            ("U64", BuiltinTypes::U64),
            ("Char", BuiltinTypes::Char),
            ("Handle", BuiltinTypes::Handle),
            ("USize", BuiltinTypes::USize),
            ("IFace", BuiltinTypes::USize),
        ]);
}

impl Type {
    pub fn new(name: String) -> Option<Self> {
        Some(Type::Builtin(*KEYWORDS.get(name.as_str())?))
    }

    pub fn as_arg(&self) -> String {
        match self {
            Self::Builtin(BuiltinTypes::Handle) => "&Handle".to_string(),
            _ => self.as_rust(),
        }
    }

    pub fn as_rust(&self) -> String {
        match self {
            Self::Builtin(bt) => {
                let s = match bt {
                    BuiltinTypes::U8 => "u8",
                    BuiltinTypes::I8 => "i8",
                    BuiltinTypes::U16 => "u16",
                    BuiltinTypes::I16 => "i16",
                    BuiltinTypes::U32 => "u32",
                    BuiltinTypes::I32 => "i32",
                    BuiltinTypes::U64 => "u64",
                    BuiltinTypes::I64 => "i64",
                    BuiltinTypes::USize => "usize",
                    BuiltinTypes::Char => "u8",
                    BuiltinTypes::Handle => "Handle",
                };

                s.to_string()
            }
            Self::Sequence { inner, count } => {
                if **inner != Self::Builtin(BuiltinTypes::Char) {
                    format!("HLVec<{inner}, {count}>", inner = inner.as_rust())
                } else {
                    format!("HLString<{count}>")
                }
            }
            Self::Struct(s) => s.name.clone(),
            Self::Enum(s) => s.name.clone(),
        }
    }

    pub fn as_wire(&self) -> String {
        match self {
            Self::Builtin(BuiltinTypes::Handle) => "usize".to_string(),
            Self::Struct(s) => format!("{}Wire", s.name.clone()),
            Self::Enum(s) => s.inner.as_wire(),
            Self::Sequence { inner, count } => {
                if **inner != Self::Builtin(BuiltinTypes::Char) {
                    format!("HLVec<{inner}, {count}>", inner = inner.as_wire())
                } else {
                    format!("HLString<{count}>")
                }
            }
            _ => self.as_rust(),
        }
    }
}
