use crate::arch::mm::layout::image_to_phys;
use crate::mm::paging::kernel_page_table::kernel_page_table;
use rtl::arch::PHYS_OFFSET;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

use alloc::vec;
use alloc::vec::Vec;

unsafe extern "C" {
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

fn kernel_sections() -> Vec<KernelSection> {
    println!("Populated kernel sections");
    vec![
        KernelSection::new(
            linker_var!(stext),
            linker_var!(etext) - linker_var!(stext),
            "Kernel text",
            MappingType::KERNEL_TEXT,
        ),
        KernelSection::new(
            linker_var!(srodata),
            linker_var!(erodata) - linker_var!(srodata),
            "Kernel rodata",
            MappingType::KERNEL_DATA_RO,
        ),
        KernelSection::new(
            linker_var!(sdata),
            linker_var!(edata) - linker_var!(sdata),
            "Kernel data",
            MappingType::KERNEL_DATA,
        ),
        KernelSection::new(
            linker_var!(sbss),
            linker_var!(ebss) - linker_var!(sbss),
            "Kernel bss",
            MappingType::KERNEL_DATA,
        ),
    ]
}

pub fn remap_kernel() {
    let array = kernel_sections();
    let mut tt = kernel_page_table();

    println!("Kernel map:");

    for i in array {
        let va = VirtAddr::from(i.start());
        let pa = image_to_phys(va);

        println!(
            "{}\t\t[0x{:x} -> 0x{:x}] (size 0x{:x})",
            i.name(),
            pa,
            va,
            i.size()
        );

        (*tt)
            .map(
                Some(MemRange::new(pa, i.size())),
                MemRange::new(va, i.size()),
                i.mapping_type(),
            )
            .expect("Failed to map kernel sections");
    }

    (*tt).walk(VirtAddr::new(0xffffffa000000000));

    println!("Remaped kernel image");
}
