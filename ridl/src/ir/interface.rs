use super::function::*;

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
}
