use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
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
    String,
    Handle,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TypeKind {
    Builtin(BuiltinTypes),
    #[allow(dead_code)]
    UserDefined(UserStruct),
}

#[derive(Clone, Debug, Hash)]
pub struct Type {
    kind: TypeKind,
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
            ("String", BuiltinTypes::String),
            ("Handle", BuiltinTypes::Handle),
        ]);
}

impl Type {
    pub fn new(name: String) -> Option<Self> {
        let kind = TypeKind::Builtin(*KEYWORDS.get(name.as_str())?);
        Some(Self { kind })
    }

    pub fn kind(&self) -> TypeKind {
        self.kind
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.kind {
            TypeKind::Builtin(bt) => {
                let s = match bt {
                    BuiltinTypes::U8 => "u8",
                    BuiltinTypes::I8 => "i8",
                    BuiltinTypes::U16 => "u16",
                    BuiltinTypes::I16 => "i16",
                    BuiltinTypes::U32 => "u32",
                    BuiltinTypes::I32 => "i32",
                    BuiltinTypes::U64 => "u64",
                    BuiltinTypes::I64 => "i64",
                    BuiltinTypes::Handle => "Handle",
                    BuiltinTypes::String => "String",
                };

                write!(f, "{s}")
            }
            _ => todo!(),
        }
    }
}
