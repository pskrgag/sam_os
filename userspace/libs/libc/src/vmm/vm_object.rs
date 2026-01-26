use crate::handle::Handle;

pub struct VmObject {
    h: Handle,
}

impl VmObject {
    pub unsafe fn new(h: Handle) -> Self {
        Self { h }
    }

    pub fn handle(&self) -> &Handle {
        &self.h
    }
}
