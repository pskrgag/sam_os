use super::argtype::Type;

#[derive(Clone)]
pub enum Argument {
    In(Type, String),
    Out(Type, String),
}

#[derive(Clone)]
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
}
