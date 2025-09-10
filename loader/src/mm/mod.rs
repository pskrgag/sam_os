use fdt::Fdt;
use page_table::{PagePerms, PageTable};
use rtl::vmm::types::{MemRange, PhysAddr, VirtAddr};

pub mod alloc;
pub mod page_table;
pub mod regions;

pub fn init(fdt: &Fdt) -> PageTable {
    regions::init(fdt);

    let mut table = PageTable::new().expect("Failed to create a page table");
    map_self_text(&mut table);
    table
}

unsafe extern "C" {
    static _text_begin: usize;
    static _text_end: usize;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {{
        #[allow(unused_unsafe)]
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            &$a as *const usize as usize
        }
    }};
}

fn map_self_text(table: &mut PageTable) {
    let text_begin = linker_var!(_text_begin);
    let text_end = linker_var!(_text_end);
    let text_size = text_end - text_begin;

    table.map_pages(
        MemRange::new(VirtAddr::new(text_begin), text_size),
        MemRange::new(PhysAddr::new(text_begin), text_size),
        PagePerms::Execute,
    );
}
