use crate::mm::{
    alloc::alloc_pages,
    page_table::{PageKind, PagePerms, PageTable},
    regions::regions,
};
use loader_protocol::{LoaderArg, VmmLayoutKind};
use rtl::arch::PAGE_SIZE;
use rtl::vmm::types::{Address, MemRange, PhysAddr, VirtAddr};

#[repr(align(0x1000))]
struct Aligned;
static INIT_TASK: &[u8] = rtl::include_bytes_align_as!(Aligned, env!("INIT_TASK_PATH"));

pub fn prepare(fdt: PhysAddr, mut arg: LoaderArg, tt: &mut PageTable) -> VirtAddr {
    let mut mmio_start = arg.get_vmm_base(VmmLayoutKind::Mmio).unwrap().0;

    arg.tt_base = tt.base().into();
    arg.fdt_base = fdt.bits();

    for dev in &mut arg.devices {
        tt.map_pages(
            MemRange::new(mmio_start, dev.size),
            MemRange::new(PhysAddr::new(dev.base), dev.size),
            PagePerms::ReadWrite,
            PageKind::Device,
        );

        dev.base = mmio_start.bits();
        mmio_start = VirtAddr::new(mmio_start.bits() + dev.size);
    }

    // Map arg page to the kernel
    let page = alloc_pages(1).unwrap();
    let arg_addr = arg.get_vmm_base(VmmLayoutKind::LoaderArg).unwrap().0;
    let image_addr = arg_addr + PAGE_SIZE;

    tt.map_pages(
        MemRange::new(arg_addr, PAGE_SIZE),
        MemRange::new(page, PAGE_SIZE),
        PagePerms::Read,
        PageKind::Normal,
    );

    tt.map_pages(
        MemRange::new(image_addr.into(), *INIT_TASK.len().round_up_page()),
        MemRange::new(
            PhysAddr::new(INIT_TASK.as_ptr() as usize),
            *INIT_TASK.len().round_up_page(),
        ),
        PagePerms::Read,
        PageKind::Normal,
    );

    // After this point is't better to not allocate any memory
    prepare_pmm(&mut arg);

    arg.init_virt_task_base = (image_addr, INIT_TASK.len());
    arg.init_phys_task_base = (INIT_TASK.as_ptr() as usize, INIT_TASK.len());
    *unsafe { &mut *(page.bits() as *mut LoaderArg) } = arg;

    arg_addr.into()
}

fn prepare_pmm(arg: &mut LoaderArg) {
    for reg in regions() {
        arg.pmm_layout
            .push(MemRange::new(reg.start, reg.count * PAGE_SIZE))
            .unwrap();
    }
}
