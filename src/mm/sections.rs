use crate::{
    kernel::locking::fake_lock::FakeLock,
    kernel::misc::kernel_offset,
    lib::collections::vector::Vector,
    linker_var,
    mm::{
        paging::{
            kernel_page_table::kernel_page_table,
            page_table::{MappingType, PageTable},
        },
        types::*,
    },
};

use core::arch::asm;

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

static KERNEL_SECTIONS: FakeLock<Vector<KernelSection>> = FakeLock::new(Vector::new());

impl KernelSection {
    pub fn new(start: usize, size: usize, name: &'static str, tp: MappingType) -> Self {
        Self {
            start: start,
            size: size,
            name: name,
            map_type: tp,
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

fn populate_kernel_sections(array: &mut Vector<KernelSection>) {
    let mut array = KERNEL_SECTIONS.get();
    let text = KernelSection::new(
        linker_var!(stext),
        linker_var!(etext) - linker_var!(stext),
        "Kernel text",
        MappingType::KernelText,
    );
    let rodata = KernelSection::new(
        linker_var!(srodata),
        linker_var!(erodata) - linker_var!(srodata),
        "Kernel rodata",
        MappingType::KernelDataRo,
    );
    let data = KernelSection::new(
        linker_var!(sdata),
        linker_var!(edata) - linker_var!(sdata),
        "Kernel data",
        MappingType::KernelData,
    );
    let bss = KernelSection::new(
        linker_var!(sbss),
        linker_var!(ebss) - linker_var!(sbss),
        "Kernel bss",
        MappingType::KernelData,
    );

    (*array).push(text);
    (*array).push(rodata);
    (*array).push(data);
    (*array).push(bss);

    println!("Populated kernel sections");
}

pub fn remap_kernel() {
    let mut array = KERNEL_SECTIONS.get();

    populate_kernel_sections(&mut *array);

    let mut tt = kernel_page_table();

    println!("Kernel map:");

    for i in &*array {
        println!(
            "{} [0x{:x} -> 0x{:x}]",
            i.name(),
            i.start() - kernel_offset(),
            i.start()
        );
        (*tt).map(
            None,
            MemRange::new(VirtAddr::from(i.start()), i.size() as usize),
            i.mapping_type(),
        );
    }

   unsafe { asm!("msr TTBR1_EL1, {}", in(reg) (*tt).base().get()) }; 
   unsafe { asm!("isb") };

   println!("Fine grained mapping enabled");

   unsafe { (array[0].start as *mut u8).write_volatile(1) };
   println!("Corrupted text");
}
