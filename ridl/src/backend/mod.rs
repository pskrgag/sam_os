use crate::ir::function::{Argument, Function};
use crate::ir::interface::Interface;
use crate::ir::IrObject;
use std::io::{Result, Write};

pub mod rust;

pub trait Backend {
    fn generate_interface_start<B: Write>(&self, out: &mut B) -> Result<()>;
    fn generate_start_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()>;
    fn generate_function_arg<B: Write>(
        &self,
        arg: &Argument,
        pos: usize,
        out: &mut B,
    ) -> Result<()>;
    fn generate_end_fuction_declaration<B: Write>(&self, arg: &Function, out: &mut B)
        -> Result<()>;
    fn generate_request_struct<B: Write>(
        &self,
        arg: &Vec<Argument>,
        names: (&str, &str),
        out: &mut B,
    ) -> Result<()>;
    fn generate_structs_inialization<B: Write>(
        &self,
        arg: &Vec<Argument>,
        names: (&str, &str),
        out: &mut B,
    ) -> Result<()>;
    fn generate_end_func<B: Write>(&self, func: &Function, out: &mut B) -> Result<()>;
    fn generate_calls<B: Write>(&self, out: &mut B) -> Result<()>;
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

pub fn compile<B: Write>(v: &Vec<Box<dyn IrObject>>, out: &mut B, lang: &str) -> bool {
    let back = match lang {
        "rust" => rust::BackendRust::default(),
        _ => {
            error!("Unknown backend requested {}", lang);
            return false;
        }
    };

    for i in v {
        if let Some(interface) = i.as_any().downcast_ref::<Interface>() {
            try_generate!(back.generate_interface_start(out), "generate iface prelude");

            for f in interface.functions() {
                let sn_in = format!("sam_request_{}_in", f.name());
                let sn_out = format!("sam_request_{}_out", f.name());
                let struct_names = (sn_in.as_str(), sn_out.as_str());

                try_generate!(
                    back.generate_request_struct(f.args(), struct_names, out),
                    "generate struct declr"
                );

                try_generate!(back.generate_start_func(f, out), "generate function start");

                for (num, arg) in f.args().iter().enumerate() {
                    try_generate!(
                        back.generate_function_arg(arg, num + 1, out),
                        "generate argument"
                    );
                }

                try_generate!(
                    back.generate_end_fuction_declaration(f, out),
                    "generate decl function end"
                );
                try_generate!(
                    back.generate_structs_inialization(f.args(), struct_names, out),
                    "generate local args init"
                );
                try_generate!(back.generate_calls(out), "generate calls");
                try_generate!(back.generate_end_func(f, out), "generate function end");
            }
        };
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

        assert!(compile(&ir.unwrap(), &mut res, "rust"));
        println!("\n{}", std::str::from_utf8(res.as_slice()).unwrap());

        // assert_eq!(std::str::from_utf8(res.as_slice()).unwrap(), expected);
    }
}
