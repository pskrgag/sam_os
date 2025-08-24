use super::argtype::Type;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Hash)]
pub enum Argument {
    In(Type, String),
    Out(Type, String),
}

#[derive(Clone, Debug)]
pub struct Function {
    uid: u64,
    name: String,
    args: Vec<Argument>,
}

impl Function {
    pub fn new(name: &[u8]) -> Self {
        let mut s = Self {
            uid: 0,
            name: std::str::from_utf8(name)
                .expect("Not utf8 source???")
                .to_owned(),
            args: Vec::new(),
        };

        let mut state = DefaultHasher::new();
        s.hash(&mut state);
        s.uid = state.finish();

        s
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

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.args.hash(state);
        self.name.hash(state);
    }
}
