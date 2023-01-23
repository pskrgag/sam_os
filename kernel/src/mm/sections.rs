use crate::{
    arch::mm::page_table::set_kernel_page_table,
    arch::{ram_base, ram_size},
    kernel::locking::fake_lock::FakeLock,
    kernel::misc::kernel_offset,
    lib::collections::vector::Vector,
    linker_var,
    mm::{
        paging::{kernel_page_table::kernel_page_table, page_table::MappingType},
        types::*,
    },
};

extern "C" {
    static stext: usize;
    static etext: usize;

    static srodata: usize;
    static erodata: usize;

    static sdata: usize;
    static edata: usize;

    static sbss: usize;
    static ebss: usize;
    static end: usize;

    static sdatapercpu: usize;
    static edatapercpu: usize;
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
    let per_cpu = KernelSection::new(
        linker_var!(sdatapercpu),
        linker_var!(edatapercpu) - linker_var!(sdatapercpu),
        "Kernel percpu",
        MappingType::KernelData,
    );

    (*array).push(text);
    (*array).push(rodata);
    (*array).push(data);
    (*array).push(bss);
    (*array).push(per_cpu);
}

pub fn remap_kernel() {
    let array = KERNEL_SECTIONS.get();

    populate_kernel_sections(&mut *array);

    let mut tt = kernel_page_table();

    println!("Kernel map:");

    for i in &*array {
        println!(
            "{}\t[0x{:x} -> 0x{:x}] (size {})",
            i.name(),
            i.start() - kernel_offset(),
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

    // For now we assume that kernel is loaded at the start of the RAM (true in case of qemu)
    // So just all RAM that we have

    let ram_start = VirtAddr::from(linker_var!(end));
    let ram_size = ram_base() as usize + ram_size() as usize - PhysAddr::from(ram_start).bits();

    (*tt)
        .map(
            None,
            MemRange::new(ram_start, ram_size),
            MappingType::KernelData,
        )
        .expect("Failed to map ram to kernel");

    unsafe { set_kernel_page_table((*tt).base()) };
    println!("Fine grained mapping enabled");
}
