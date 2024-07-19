use crate::ir::function::Function;
use crate::ir::interface::Interface;
use crate::ir::IrObject;
use std::io::{Result, Write};

pub mod rust;

pub trait Backend {
    fn generate_file_start<B: Write>(&self, out: &mut B) -> Result<()>;
    fn generate_transport_init<B: Write>(&self, out: &mut B) -> Result<()>;
    fn generate_start_transport_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()>;
    fn generate_end_fuction_declaration<B: Write>(&self, arg: &Function, out: &mut B)
        -> Result<()>;
    fn generate_request_struct<B: Write>(&self, f: &Function, out: &mut B) -> Result<()>;
    fn generate_end_func<B: Write>(&self, out: &mut B) -> Result<()>;
    fn generate_calls<B: Write>(&self, f: &Function, out: &mut B) -> Result<()>;
    fn generate_server_event_loop<B: Write>(&self, f: &Vec<Function>, out: &mut B) -> Result<()>;
    fn generate_request_struct_server<B: Write>(&self, f: &Function, out: &mut B) -> Result<()>;
}

macro_rules! try_generate {
    ($t:expr, $msg:expr) => {
        match $t {
            Err(_) => {
                error!("Failed to {}", $msg);
                return false;
            }
            _ => {}
        }
    };
}

pub fn compile_transport<B: Write>(v: &Vec<Box<dyn IrObject>>, out: &mut B, lang: &str) -> bool {
    let back = match lang {
        "rust" => rust::BackendRust::default(),
        _ => {
            error!("Unknown backend requested {}", lang);
            return false;
        }
    };

    try_generate!(back.generate_file_start(out), "generate iface prelude");
    try_generate!(back.generate_transport_init(out), "transport init");

    for i in v {
        if let Some(interface) = i.as_any().downcast_ref::<Interface>() {
            for f in interface.functions() {
                try_generate!(
                    back.generate_request_struct(f, out),
                    "generate struct declr"
                );

                try_generate!(
                    back.generate_start_transport_func(f, out),
                    "generate function start"
                );

                try_generate!(
                    back.generate_end_fuction_declaration(f, out),
                    "generate decl function end"
                );
                try_generate!(back.generate_calls(f, out), "generate calls");
                try_generate!(back.generate_end_func(out), "generate function end");
            }
        };
    }

    true
}

pub fn compile_server<B: Write>(v: &Vec<Box<dyn IrObject>>, out: &mut B, lang: &str) -> bool {
    let back = match lang {
        "rust" => rust::BackendRust::default(),
        _ => {
            error!("Unknown backend requested {}", lang);
            return false;
        }
    };

    try_generate!(back.generate_file_start(out), "generate iface prelude");
    try_generate!(back.generate_transport_init(out), "transport init");

    for i in v {
        if let Some(interface) = i.as_any().downcast_ref::<Interface>() {
            for f in interface.functions() {
                try_generate!(
                    back.generate_request_struct_server(f, out),
                    "generate struct declr"
                );
            }

            try_generate!(
                back.generate_server_event_loop(&interface.functions(), out),
                "generate struct declr"
            );
        }
    }

    true
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error_reporter;
    use crate::frontend::lexer::Lexer;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_simple_func_rust() {
        let text = "interface { Test(out I32 a); }";
        let lexer = Lexer::new(text.as_bytes());
        let reporter = error_reporter::ErrorReporter::new(text.as_bytes());
        let mut parser = Parser::new(lexer, &reporter);
        let expected = "pub struct sam_request_Test {\n    \
                        pub a: i32,\n\
                        }\n\
                        pub fn Test(a: &mut i32) -> Result<usize>\n";

        let ir = parser.parse();
        assert!(ir.is_some());

        let mut res = Vec::new();

        assert!(compile_transport(&ir.unwrap(), &mut res, "rust"));
        println!("\n{}", std::str::from_utf8(res.as_slice()).unwrap());

        assert_eq!(std::str::from_utf8(res.as_slice()).unwrap(), expected);
    }
}
