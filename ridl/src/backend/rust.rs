use super::Backend;
use crate::ir::argtype::{BuiltinTypes, Type, TypeKind};
use crate::ir::function::{Argument, Function};
use std::io::{Result, Write};

#[derive(Default)]
pub struct BackendRust {}

const LOCAL_IN_STRUCT_NAME: &str = "to_call_in";
const LOCAL_OUT_STRUCT_NAME: &str = "to_call_out";
const SERVER_HANDLE: &str = "SERVER_HANDLE";
const REQUEST_HEADER_STRUCT_NAME: &str = "RequestHeader";
const EVENT_LOOP_ARG_NAME: &str = "cbs";
const DERIVES: &str = "#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]\n#[repr(C, packed)]#[allow(private_interfaces)]\n";

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

    fn format_inout_structs(f: &Function) -> (String, String) {
        (
            format!("sam_request_{}_in", f.name()),
            format!("sam_request_{}_out", f.name()),
        )
    }

    fn generate_start_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()> {
        write!(out, "pub fn {}(", func.name())
    }

    fn generate_server_virt_table(&self, func: &Vec<Function>) -> String {
        let mut s = String::new();

        for i in func {
            let names = Self::format_inout_structs(i);
            s.push_str(format!("pub cb_{}: fn({}) -> {},\n", i.name(), names.0, names.1).as_str());
        }

        s
    }

    fn generate_server_union(&self, func: &Vec<Function>) -> String {
        let mut s = String::new();

        for i in func {
            let names = Self::format_inout_structs(i);
            s.push_str(format!("req_{}: {},\n", i.name(), names.0).as_str());
        }

        s
    }

    fn generate_server_event_loop_match_arms(&self, func: &Vec<Function>) -> String {
        let mut s = String::new();

        for i in func {
            let names = Self::format_inout_structs(i);
            s.push_str(
                format!(
                    "
                    {} => {{
                        let arg: *const {} = 
                            core::mem::transmute(
                                buff.as_ptr().offset(core::mem::size_of::<{REQUEST_HEADER_STRUCT_NAME}>() as isize)
                            );
                        let res = ({EVENT_LOOP_ARG_NAME}.cb_{})(*arg);
                        {SERVER_HANDLE}.as_ref().unwrap().send_data(port, bytemuck::bytes_of(&res));
                    }}
                            ",
                    i.uid(),
                    names.0,
                    i.name(),
                )
                .as_str(),
            );
        }

        s
    }

    fn generate_call(&self) -> String {
        format!(
            "    unsafe {{
        let r = if core::mem::size_of_val(&{LOCAL_OUT_STRUCT_NAME}) != 0 {{ Some(bytemuck::bytes_of_mut(&mut {LOCAL_OUT_STRUCT_NAME})) }} else {{ None }};
        {SERVER_HANDLE}.as_ref().unwrap().call(bytemuck::bytes_of(&{LOCAL_IN_STRUCT_NAME}), r);
        }}")
    }

    fn generate_in_call(&self) -> String {
        format!(
            "    unsafe {{
        if core::mem::size_of_val(&{LOCAL_OUT_STRUCT_NAME}) != 0 {{
            {SERVER_HANDLE}.as_ref().unwrap().receive_data(bytemuck::bytes_of_mut(&mut {LOCAL_OUT_STRUCT_NAME}));
        }}
    }}")
    }

    fn generate_request_structs(&self, f: &Function, header: bool) -> String {
        let header = if header {
            format!("    pub hdr: {REQUEST_HEADER_STRUCT_NAME},")
        } else {
            "".to_owned()
        };
        let names = Self::format_inout_structs(f);
        let arg = f.args();

        let mut s = format!("{DERIVES}pub struct {} {{\n{header}", names.0);

        for i in arg {
            match i {
                Argument::In(t, name) => {
                    s.push_str(format!("    pub {name}: {},", self.type_to_str(t)).as_str());
                }
                _ => {}
            }
        }

        s.push_str(format!("}}\n").as_str());
        s.push_str(format!("{DERIVES}pub struct {} {{\n", names.1).as_str());

        for i in arg {
            match i {
                Argument::Out(t, name) => {
                    s.push_str(format!("    pub {name}: {},", self.type_to_str(t)).as_str());
                }
                _ => {}
            }
        }

        s.push_str(format!("}}\n").as_str());
        s
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

    fn generate_start_transport_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()> {
        self.generate_start_func(func, out)
    }

    // fn generate_start_server_func<B: Write>(
    //     &self,
    //     func: &Function,
    //     names: (&str, &str),
    //     out: &mut B,
    // ) -> Result<()> {
    //     let cb_generic = format!("F: Fn({}) -> {}", names.0, names.1);
    //     self.generate_start_func(func, out, cb_generic.as_str(), "cb: F")
    // }

    fn generate_end_fuction_declaration<B: Write>(&self, _f: &Function, out: &mut B) -> Result<()> {
        writeln!(out, ") -> Result<usize, usize> {{")
    }

    fn generate_calls<B: Write>(&self, out: &mut B) -> Result<()> {
        writeln!(out, "{}", self.generate_call(),)
    }

    fn generate_request_struct<B: Write>(&self, f: &Function, out: &mut B) -> Result<()> {
        write!(out, "{}", self.generate_request_structs(f, true))
    }

    fn generate_request_struct_server<B: Write>(&self, f: &Function, out: &mut B) -> Result<()> {
        write!(out, "{}", self.generate_request_structs(f, false))
    }

    fn generate_structs_inialization<B: Write>(&self, f: &Function, out: &mut B) -> Result<()> {
        let arg = f.args();
        let header = format!(
            "        hdr: {REQUEST_HEADER_STRUCT_NAME} {{ num: {} }},",
            f.uid()
        );
        let names = Self::format_inout_structs(f);

        write!(
            out,
            "    let {LOCAL_IN_STRUCT_NAME} = {} {{\n{header}\n",
            names.0
        )?;

        for i in arg {
            match i {
                Argument::In(_, name) => {
                    write!(out, "        {name}: *{name},\n")?;
                }
                _ => {}
            }
        }

        write!(out, "    }};\n")?;
        write!(
            out,
            "    let mut {LOCAL_OUT_STRUCT_NAME} = {}::zeroed();\n",
            names.1
        )
    }

    fn generate_end_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()> {
        for i in func.args() {
            match i {
                Argument::Out(_, name) => {
                    writeln!(out, "*{name} = {LOCAL_OUT_STRUCT_NAME}.{name};")?;
                }
                _ => {}
            }
        }

        writeln!(out, "    Ok(0)")?;
        writeln!(out, "}}")
    }

    fn generate_file_start<B: Write>(&self, out: &mut B) -> Result<()> {
        let req = format!(
            "{DERIVES}struct {REQUEST_HEADER_STRUCT_NAME} {{
    pub num: u64,
}}"
        );
        writeln!(out, "use libc::port::Port;")?;
        writeln!(out, "use rtl::handle::*;")?;
        writeln!(out, "use bytemuck;")?;
        writeln!(out, "use bytemuck::Zeroable;")?;
        writeln!(out, "")?;
        writeln!(out, "static mut {SERVER_HANDLE}: Option<Port> = None;\n")?;
        writeln!(out, "{req}\n")
    }

    fn generate_transport_init<B: Write>(&self, out: &mut B) -> Result<()> {
        writeln!(
            out,
            "pub fn init(h: Handle) {{
    if h != HANDLE_INVALID {{
        unsafe {{ {SERVER_HANDLE} = Some(Port::new(h)); }}
    }}
}}\n"
        )
    }

    fn generate_server_event_loop<B: Write>(&self, f: &Vec<Function>, out: &mut B) -> Result<()> {
        writeln!(
            out,
            "
pub struct ServerVirtTable {{
    {}}}

pub fn start_server({EVENT_LOOP_ARG_NAME}: ServerVirtTable, p: Port) -> ! {{
    unsafe {{
        {SERVER_HANDLE} = Some(p);
    }}

    loop {{
        unsafe {{
            union Buffer___ {{
                {}
            }};

            let mut buff = [0u8; core::mem::size_of::<Buffer___>() + core::mem::size_of::<{REQUEST_HEADER_STRUCT_NAME}>()];
            let header: *const {REQUEST_HEADER_STRUCT_NAME} = core::mem::transmute(buff.as_ptr());

            let port = {SERVER_HANDLE}.as_ref().unwrap().receive_data(bytemuck::bytes_of_mut(&mut buff)).unwrap().unwrap();

            let n = (*header).num;
            match (*header).num {{
                {}
                _ => {{ panic!() }}
            }};
        }}
    }};
}}\n",
            self.generate_server_virt_table(f),
            self.generate_server_union(f),
            self.generate_server_event_loop_match_arms(f),
        )
    }
}
