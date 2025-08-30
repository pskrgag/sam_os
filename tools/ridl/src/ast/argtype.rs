use std::collections::HashMap;
use strum_macros::Display;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct UserStruct {}

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
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Builtin(BuiltinTypes),
    Sequence {
        inner: Box<Type>,
        count: usize,
    },
    #[allow(dead_code)]
    UserDefined(UserStruct),
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
        ]);
}

impl Type {
    pub fn new(name: String) -> Option<Self> {
        Some(Type::Builtin(*KEYWORDS.get(name.as_str())?))
    }

    pub fn as_arg(&self) -> String {
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
                    BuiltinTypes::Char => "u8",
                    BuiltinTypes::Handle => "&Handle",
                };

                s.to_string()
            }
            Self::Sequence { inner, .. } => {
                if **inner != Self::Builtin(BuiltinTypes::Char) {
                    format!("&[{}]", inner.as_wire())
                } else {
                    // Special case, since &[char] != &str in rust
                    "&str".to_string()
                }
            }
            _ => todo!(),
        }
    }

    pub fn as_public(&self) -> String {
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
                    BuiltinTypes::Char => "u8",
                    BuiltinTypes::Handle => "Handle",
                };

                s.to_string()
            }
            Self::Sequence { inner, .. } => {
                if **inner != Self::Builtin(BuiltinTypes::Char) {
                    format!("Vec<{}>", inner.as_wire())
                } else {
                    "String".to_string()
                }
            }
            _ => todo!(),
        }
    }

    pub fn as_wire(&self) -> String {
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
                    BuiltinTypes::Char => "u8",
                    BuiltinTypes::Handle => "usize",
                };

                s.to_string()
            }
            Self::Sequence { inner, count } => format!("(usize, [{}; {count}])", inner.as_wire()),
            _ => todo!(),
        }
    }
}
