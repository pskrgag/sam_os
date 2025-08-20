use super::function::*;

#[derive(Debug)]
pub struct Interface {
    funcs: Vec<Function>,
    name: String,
}

impl Interface {
    pub fn new(name: String) -> Self {
        Self { funcs: Vec::new(), name }
    }

    pub fn add_func(&mut self, f: Function) {
        self.funcs.push(f);
    }

    pub fn functions(&self) -> &Vec<Function> {
        &self.funcs
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
