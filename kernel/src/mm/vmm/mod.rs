use loader_protocol::LoaderArg;

pub mod layout;
pub mod vmo;
pub mod vms;
pub mod vma_list;

pub fn init(prot: &LoaderArg) {
    layout::init(prot);
}
