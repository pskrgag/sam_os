use super::argtype::Struct;
use super::interface::Interface;

#[derive(Debug)]
pub struct Module {
    name: String,
    mods: Vec<Interface>,
    structs: Vec<Struct>,
}

impl Module {
    pub fn new(name: String, mods: Vec<Interface>, structs: Vec<Struct>) -> Self {
        Self {
            mods,
            structs,
            name,
        }
    }

    pub fn interfaces(&self) -> &Vec<Interface> {
        &self.mods
    }

    pub fn structs(&self) -> &Vec<Struct> {
        &self.structs
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
