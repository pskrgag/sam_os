use super::function::*;
use ir_lib::ir;

#[derive(Debug, ir)]
pub struct Interface {
    funcs: Vec<Function>,
    name: String,
}

impl Interface {
    pub fn new(name: String) -> Self {
        Self {
            funcs: Vec::new(),
            name,
        }
    }

    pub fn add_func(&mut self, f: Function) {
        self.funcs.push(f);
    }

    pub fn functions(&self) -> &Vec<Function> {
        &self.funcs
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}
