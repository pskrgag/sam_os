use crate::{linker_var, mm::paging::kernel_page_table::kernel_page_table};
use rtl::arch::PHYS_OFFSET;
use rtl::locking::fake_lock::FakeLock;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

use alloc::vec::Vec;

extern "C" {
    static stext: usize;
    static etext: usize;

    static srodata: usize;
    static erodata: usize;

    static sdata: usize;
    static edata: usize;

    static sbss: usize;
    static ebss: usize;
}

pub struct KernelSection {
    start: usize,
    size: usize,
    name: &'static str,
    map_type: MappingType,
}

static KERNEL_SECTIONS: FakeLock<Vec<KernelSection>> = FakeLock::new(Vec::new());

impl KernelSection {
    pub fn new(start: usize, size: usize, name: &'static str, map_type: MappingType) -> Self {
        Self {
            start,
            size,
            name,
            map_type,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn mapping_type(&self) -> MappingType {
        self.map_type
    }
}

fn populate_kernel_sections(array: &mut Vec<KernelSection>) {
    let text = KernelSection::new(
        linker_var!(stext),
        linker_var!(etext) - linker_var!(stext),
        "Kernel text",
        MappingType::KERNEL_TEXT,
    );
    let rodata = KernelSection::new(
        linker_var!(srodata),
        linker_var!(erodata) - linker_var!(srodata),
        "Kernel rodata",
        MappingType::KERNEL_DATA_RO,
    );
    let data = KernelSection::new(
        linker_var!(sdata),
        linker_var!(edata) - linker_var!(sdata),
        "Kernel data",
        MappingType::KERNEL_DATA,
    );
    let bss = KernelSection::new(
        linker_var!(sbss),
        linker_var!(ebss) - linker_var!(sbss),
        "Kernel bss",
        MappingType::KERNEL_DATA,
    );

    (*array).push(text);
    (*array).push(rodata);
    (*array).push(data);
    (*array).push(bss);

    println!("Populated kernel sections");
}

pub fn remap_kernel() {
    let array = KERNEL_SECTIONS.get();

    populate_kernel_sections(&mut *array);

    let mut tt = kernel_page_table();

    println!("Kernel map:");

    for i in &*array {
        println!(
            "{}\t\t[0x{:x} -> 0x{:x}] (size 0x{:x})",
            i.name(),
            i.start() - PHYS_OFFSET,
            i.start(),
            i.size()
        );

        (*tt)
            .map(
                None,
                MemRange::new(VirtAddr::from(i.start()), i.size() as usize),
                i.mapping_type(),
            )
            .expect("Failed to map kernel sections");
    }

    println!("Remaped kernel image");
}
