use super::interface::Interface;

#[derive(Debug)]
pub struct Module {
    mods: Vec<Interface>,
}

impl Module {
    pub fn new() -> Self {
        Self { mods: Vec::new() }
    }

    pub fn add_interface(&mut self, f: Interface) {
        self.mods.push(f);
    }

    pub fn interfaces(&self) -> &Vec<Interface> {
        &self.mods
    }
}
