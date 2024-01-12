use super::function::*;
use ir_lib::ir;

#[derive(Debug, ir)]
pub struct Interface {
    funcs: Vec<Function>,
}

impl Interface {
    pub fn new() -> Self {
        Self { funcs: Vec::new() }
    }

    pub fn add_func(&mut self, f: Function) {
        self.funcs.push(f);
    }

    pub fn functions(&self) -> &Vec<Function> {
        &self.funcs
    }
}
