use super::argtype::Type;
use ir_lib::ir;

#[derive(Clone, Debug)]
pub enum Argument {
    In(Type, String),
    Out(Type, String),
}

#[derive(Clone, ir, Debug)]
pub struct Function {
    name: String,
    args: Vec<Argument>,
}

impl Function {
    pub fn new(name: &[u8]) -> Self {
        Self {
            name: std::str::from_utf8(name)
                .expect("Not utf8 source???")
                .to_owned(),
            args: Vec::new(),
        }
    }

    pub fn add_arg(&mut self, arg: Argument) {
        self.args.push(arg);
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn args(&self) -> &Vec<Argument> {
        &self.args
    }
}
