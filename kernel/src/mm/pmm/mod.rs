use loader_protocol::LoaderArg;

pub mod page;
pub mod page_alloc;
pub mod page_list;
pub mod phys_layout;

pub fn init(prot: &LoaderArg) {
    phys_layout::init(prot);
    page_alloc::init();
}
