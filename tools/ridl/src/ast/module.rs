use super::{argtype::Type, interface::Interface};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Module {
    aliases: HashMap<String, Type>,
    mods: Vec<Interface>,
}

impl Module {
    pub fn new(aliases: HashMap<String, Type>, mods: Vec<Interface>) -> Self {
        Self { aliases, mods }
    }

    pub fn interfaces(&self) -> &Vec<Interface> {
        &self.mods
    }
}
