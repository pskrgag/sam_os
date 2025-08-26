use super::{interface::Interface};

#[derive(Debug)]
pub struct Module {
    mods: Vec<Interface>,
}

impl Module {
    pub fn new(mods: Vec<Interface>) -> Self {
        Self { mods }
    }

    pub fn interfaces(&self) -> &Vec<Interface> {
        &self.mods
    }
}
