use super::argtype::{Enum, Struct};
use super::interface::Interface;

#[derive(Debug)]
pub struct Module {
    name: String,
    mods: Vec<Interface>,
    structs: Vec<Struct>,
    enums: Vec<Enum>,
}

impl Module {
    pub fn new(name: String, mods: Vec<Interface>, structs: Vec<Struct>, enums: Vec<Enum>) -> Self {
        Self {
            mods,
            structs,
            name,
            enums,
        }
    }

    pub fn interfaces(&self) -> &Vec<Interface> {
        &self.mods
    }

    pub fn structs(&self) -> &Vec<Struct> {
        &self.structs
    }

    pub fn enums(&self) -> &Vec<Enum> {
        &self.enums
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
