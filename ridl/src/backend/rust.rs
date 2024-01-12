use super::Backend;
use crate::ir::argtype::{BuiltinTypes, Type, TypeKind};
use crate::ir::function::{Argument, Function};
use std::io::{Result, Write};

#[derive(Default)]
pub struct BackendRust {}

const LOCAL_IN_STRUCT_NAME: &str = "to_call_in";
const LOCAL_OUT_STRUCT_NAME: &str = "to_call_out";
const SERVER_HANDLE: &str = "SERVER_HANDLE";

impl BackendRust {
    fn type_to_str(&self, t: &Type) -> &str {
        match t.kind() {
            TypeKind::UserDefined(_) => todo!(),
            TypeKind::Builtin(b) => match b {
                BuiltinTypes::U8 => "u8",
                BuiltinTypes::I8 => "i8",
                BuiltinTypes::U16 => "u16",
                BuiltinTypes::I16 => "i16",
                BuiltinTypes::U32 => "u32",
                BuiltinTypes::I32 => "i32",
                BuiltinTypes::U64 => "u64",
                BuiltinTypes::I64 => "i64",
                BuiltinTypes::String => "String",
                BuiltinTypes::Handle => "Handle",
            },
        }
    }
}

impl Backend for BackendRust {
    fn generate_function_arg<B: Write>(
        &self,
        arg: &Argument,
        pos: usize,
        out: &mut B,
    ) -> Result<()> {
        if pos != 0 {
            write!(out, ", ")?;
        }

        match arg {
            Argument::In(t, name) => {
                write!(out, "{name}: &{}", self.type_to_str(t))
            }
            Argument::Out(t, name) => {
                write!(out, "{name}: &mut {}", self.type_to_str(t))
            }
        }
    }

    fn generate_start_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()> {
        write!(out, "pub fn {}(server_handle: Handle", func.name())
    }

    fn generate_end_fuction_declaration<B: Write>(&self, _f: &Function, out: &mut B) -> Result<()> {
        writeln!(out, ") -> Result<usize, usize> {{")
    }

    fn generate_calls<B: Write>(&self, out: &mut B) -> Result<()> {
        writeln!(
            out,
            "unsafe {{
            {SERVER_HANDLE}.as_ref().unwrap().send_data(bytemuck::bytes_of(&{LOCAL_IN_STRUCT_NAME}))
            }}"
        )?;
        writeln!(
            out,
            "unsafe {{ \
            if core::mem::size_of_val(&{LOCAL_OUT_STRUCT_NAME}) != 0 {{\n \
            {SERVER_HANDLE}.as_ref().unwrap().receive_data(bytemuck::bytes_of_mut(&mut {LOCAL_OUT_STRUCT_NAME})) \
            }}\n
            }}"
        )
    }

    fn generate_request_struct<B: Write>(
        &self,
        arg: &Vec<Argument>,
        names: (&str, &str),
        out: &mut B,
    ) -> Result<()> {
        writeln!(
            out,
            "#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]\n#[repr(C)]\npub struct {} {{",
            names.0
        )?;

        for i in arg {
            match i {
                Argument::In(t, name) => {
                    writeln!(out, "    pub {name}: {},", self.type_to_str(t))?;
                }
                _ => {}
            }
        }

        writeln!(out, "}}")?;
        writeln!(
            out,
            "#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]\n#[repr(C)]\npub struct {} {{",
            names.1
        )?;

        for i in arg {
            match i {
                Argument::Out(t, name) => {
                    writeln!(out, "    pub {name}: {},", self.type_to_str(t))?;
                }
                _ => {}
            }
        }

        writeln!(out, "}}")
    }

    fn generate_structs_inialization<B: Write>(
        &self,
        arg: &Vec<Argument>,
        names: (&str, &str),
        out: &mut B,
    ) -> Result<()> {
        write!(out, "let {LOCAL_IN_STRUCT_NAME} = {} {{\n", names.0)?;

        for i in arg {
            match i {
                Argument::In(_, name) => {
                    write!(out, "{name}: *{name},\n")?;
                }
                _ => {}
            }
        }

        write!(out, "}};\n")?;
        write!(out, "let mut {LOCAL_OUT_STRUCT_NAME} = {} {{\n", names.1)?;

        for i in arg {
            match i {
                Argument::Out(_, name) => {
                    write!(out, "{name}: *{name},\n")?;
                }
                _ => {}
            }
        }

        writeln!(out, "}};")
    }

    fn generate_end_func<B: Write>(&self, _func: &Function, out: &mut B) -> Result<()> {
        writeln!(out, "Ok(0)")?;
        writeln!(out, "}}")
    }

    fn generate_interface_start<B: Write>(&self, out: &mut B) -> Result<()> {
        writeln!(out, "use libc::port::Port;")?;
        writeln!(out, "use rtl::handle::*;")?;
        writeln!(out, "use bytemuck;")?;
        writeln!(out, "")?;
        writeln!(out, "static mut {SERVER_HANDLE}: Option<Port> = None;\n")
    }
}
