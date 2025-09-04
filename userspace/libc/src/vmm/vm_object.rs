use crate::handle::Handle;

pub struct VmObject {
    h: Handle,
}

impl VmObject {
    pub fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }
}
