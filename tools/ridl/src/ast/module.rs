use super::argtype::Struct;
use super::interface::Interface;

#[derive(Debug)]
pub struct Module {
    mods: Vec<Interface>,
    structs: Vec<Struct>,
}

impl Module {
    pub fn new(mods: Vec<Interface>, structs: Vec<Struct>) -> Self {
        Self { mods, structs }
    }

    pub fn interfaces(&self) -> &Vec<Interface> {
        &self.mods
    }

    pub fn structs(&self) -> &Vec<Struct> {
        &self.structs
    }
}
