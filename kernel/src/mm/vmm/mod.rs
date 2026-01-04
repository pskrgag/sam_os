use loader_protocol::LoaderArg;

pub mod layout;
pub mod vma_list;
pub mod vmo;
pub mod vms;

pub fn init(prot: &LoaderArg) {
    layout::init(prot);
}
