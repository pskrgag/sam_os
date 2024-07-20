use super::PrintContext;
use crate::ir::function::Function;
use crate::ir::interface::Interface;
use std::io::{Result, Write};
use std::rc::Rc;

pub struct RustBackend<'a, B: Write> {
    context: PrintContext<'a, B>,
}

impl<'a, B: Write> RustBackend<'a, B> {
    pub fn new(out: &'a mut B) -> Self {
        Self {
            context: PrintContext::new(4, out),
        }
    }

    fn emit_prelude(&mut self) {
        self.context.println("use bytemuck::*;");
        self.context.println("");
    }

    pub fn compile_transport(&mut self, v: &Interface) -> Result<()> {
        todo!()
    }

    fn emit_server_function(&mut self, v: &Function) {
        self.context.print(format!("pub fn {}(", v.name()));

        for i in v.args() {

        }
    }

    pub fn compile_server(&mut self, v: &Interface) -> Result<()> {
        self.emit_prelude();

        self.context
            .print(format!("pub struct {}server {{", v.name()));

        for i in v.functions() {
            self.context.inc_indent();

            self.emit_server_function(i);

            self.context.dec_indent();
        }

        self.context.print(format!("}}"));
        todo!()
    }
}
