use super::Backend;
use crate::ir::argtype::{BuiltinTypes, Type, TypeKind};
use crate::ir::function::{Argument, Function};
use std::io::{Result, Write};

#[derive(Default)]
pub struct BackendRust {}

const SERVER_HANDLE: &str = "SERVER_HANDLE";
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
                BuiltinTypes::String => "ArenaPtr",
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

    fn generate_req_union_body(&self, f: &Vec<Function>) -> String {
        let mut s = String::new();
        for i in f {
            let names = Self::format_inout_structs(i);
            s.push_str(format!("pub req_{}: {},\n", i.name(), names.0).as_str());
        }
        s
    }

    fn generate_resp_union_body(&self, f: &Vec<Function>) -> String {
        let mut s = String::new();
        for i in f {
            let names = Self::format_inout_structs(i);
            s.push_str(format!("pub req_{}: {},\n", i.name(), names.1).as_str());
        }
        s
    }

    fn generate_start_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()> {
        write!(out, "pub fn sam_{}(", func.name())
    }

    fn generate_server_virt_table(&self, func: &Vec<Function>) -> String {
        let mut s = String::new();

        for i in func {
            let names = Self::format_inout_structs(i);
            s.push_str(format!("pub cb_{}: fn({}, req_arena: &MessageArena, resp_arena: &mut MessageArena) -> Result<{}, ErrorType>,\n", i.name(), names.0, names.1).as_str());
        }

        s
    }

    fn generate_server_event_loop_match_arms(&self, func: &Vec<Function>) -> String {
        let mut s = String::new();

        for i in func {
            s.push_str(
                format!(
                    "
                    {} => {{
                        let arg = unsafe {{ &mut request.req_{} }};

                        {};

                        match (self.cb_{})(*arg, req_arena, resp_arena) {{
                            Ok(rr) => {{ 
                                response.req_{} = rr;
                                {}
                            }},
                            Err(err) => response.req_{}.error = err,
                        }};

                    }}
                            ",
                    i.uid(),
                    i.name(),
                    self.generate_handle_transfer_server(i, true),
                    i.name(),
                    i.name(),
                    self.generate_handle_transfer_server(i, false),
                    i.name(),
                )
                .as_str(),
            );
        }

        s
    }

    fn generate_request_structs(&self, f: &Function) -> String {
        let names = Self::format_inout_structs(f);
        let arg = f.args();

        let mut s = format!("{DERIVES}pub struct {} {{\n", names.0);

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

        s.push_str(format!("    pub error: ErrorType,").as_str());

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

    fn generate_handle_transfer_client(&self, f: &Function, iin: bool) -> String {
        let mut s = String::new();
        let mut index = 0;

        if !iin {
            let names = Self::format_inout_structs(f);

            s.push_str(format!("let h = ipc.handles();\n").as_str());
            s.push_str(
                format!(
                    "let mut resp_ = resp_arena.read::<{}>(ArenaPtr::request_ptr::<{}>()).unwrap();
                     let error = resp_.error;
                     if error != 0.into() {{
                        return Err(error.into());
                     }}
                    ",
                    names.1, names.1
                )
                .as_str(),
            );
        }

        for i in f.args() {
            if !iin {
                match i {
                    Argument::Out(tp, name) => {
                        if tp.kind() == TypeKind::Builtin(crate::ir::argtype::BuiltinTypes::Handle)
                        {
                            s.push_str(format!("resp_.{} = h[{index}];\n", name).as_str());

                            index += 1;
                        }
                    }
                    _ => {}
                }
            } else {
                match i {
                    Argument::In(tp, name) => {
                        if tp.kind() == TypeKind::Builtin(crate::ir::argtype::BuiltinTypes::Handle)
                        {
                            s.push_str(
                                format!("req.{} = ipc.add_handle(req.{});\n", name, name).as_str(),
                            );

                            index += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        s
    }

    fn generate_handle_transfer_server(&self, f: &Function, iin: bool) -> String {
        let mut s = String::new();
        let mut index = 0;

        if iin {
            s.push_str(format!("let h = in_ipc.handles();\n").as_str());
        }

        for i in f.args() {
            if !iin {
                match i {
                    Argument::Out(tp, name) => {
                        if tp.kind() == TypeKind::Builtin(crate::ir::argtype::BuiltinTypes::Handle)
                        {
                            s.push_str(
                                format!(
                            "response.req_{}.{} = out_ipc.add_handle(unsafe {{ response.req_{}.{} }})",
                            f.name(),
                            name,
                            f.name(),
                            name
                        )
                                .as_str(),
                            );
                        }
                    }
                    _ => {}
                }
            } else {
                match i {
                    Argument::In(tp, name) => {
                        if tp.kind() == TypeKind::Builtin(crate::ir::argtype::BuiltinTypes::Handle)
                        {
                            s.push_str(format!("    arg.{} = h[{index}];\n", name).as_str());
                            index += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        s
    }
}

impl Backend for BackendRust {
    fn generate_start_transport_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()> {
        self.generate_start_func(func, out)?;
        let names = Self::format_inout_structs(func);
        write!(
            out,
            "req: &mut {}, req_arena: &MessageArena, resp: &mut {}, resp_arena: &mut MessageArena",
            names.0, names.1
        )
    }

    fn generate_end_fuction_declaration<B: Write>(&self, _f: &Function, out: &mut B) -> Result<()> {
        writeln!(out, ") -> Result<usize, usize> {{")
    }

    fn generate_calls<B: Write>(&self, f: &Function, out: &mut B) -> Result<()> {
        writeln!(
            out,
            "
    let mut ipc = IpcMessage::new();

    ipc.set_out_arena(req_arena.as_slice_allocated());

    // if let Some(arena) = resp_arena {{
        ipc.set_in_arena(resp_arena.as_slice());
    // }}

    ipc.set_mid({});

    {}

    unsafe {{ {SERVER_HANDLE}.as_ref().unwrap().call(&mut ipc) }}?;
    {}
    *resp = resp_;
",
            f.uid(),
            self.generate_handle_transfer_client(f, true),
            self.generate_handle_transfer_client(f, false)
        )
    }

    fn generate_request_struct<B: Write>(&self, f: &Function, out: &mut B) -> Result<()> {
        write!(out, "{}", self.generate_request_structs(f))
    }

    fn generate_request_struct_server<B: Write>(&self, f: &Function, out: &mut B) -> Result<()> {
        self.generate_request_struct(f, out)
    }

    fn generate_end_func<B: Write>(&self, out: &mut B) -> Result<()> {
        writeln!(out, "    Ok(0)")?;
        writeln!(out, "}}")
    }

    fn generate_file_start<B: Write>(&self, out: &mut B) -> Result<()> {
        writeln!(out, "use libc::port::Port;")?;
        writeln!(out, "use rtl::handle::*;")?;
        writeln!(out, "use bytemuck::*;")?;
        writeln!(out, "use rtl::error::*;")?;
        writeln!(out, "use rtl::ipc::message::*;")?;
        writeln!(out, "use ridlrt::arena::*;")?;
        writeln!(out, "use libc::port::*;")?;
        writeln!(out, "")?;
        writeln!(out, "static mut {SERVER_HANDLE}: Option<Port> = None;\n")
    }

    fn generate_transport_init<B: Write>(&self, out: &mut B) -> Result<()> {
        writeln!(
            out,
            "pub fn sam_transport_init(h: Handle) {{
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
    use ridlrt::server::Dispatcher;
    use ridlrt::arena::MessageArena;

    pub struct Disp {{
        {}
    }}

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union RequestUnion {{
        {}
    }}

    #[derive(Copy, Clone, Zeroable)]
    #[repr(C)]
    pub union ResponseUnion {{
        {}
    }}

    impl Dispatcher for Disp {{
        type DispatchReq = RequestUnion;
        type DispatchResp = ResponseUnion;

        fn dispatch(
            &self,
            in_ipc: &IpcMessage,
            out_ipc: &mut IpcMessage,
            request: &mut Self::DispatchReq,
            req_arena: &MessageArena,
            response: &mut Self::DispatchResp,
            resp_arena: &mut MessageArena,
        ) {{
            match in_ipc.mid() {{
                {}
                _ => panic!(),
            }}
        }}
    }}
        ",
            self.generate_server_virt_table(f),
            self.generate_req_union_body(f),
            self.generate_resp_union_body(f),
            self.generate_server_event_loop_match_arms(f),
        )
    }
}
