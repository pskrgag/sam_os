use rtl::vmm::types::{MemRange, VirtAddr};

pub struct Ecam {
    range: MemRange<VirtAddr>,
}

impl Ecam {
    pub fn new(range: MemRange<VirtAddr>) -> Self {
        Self { range }
    }

    pub fn enumerate(&self) {

    }
}
