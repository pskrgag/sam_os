pub mod kernel_page_table;
pub mod page_table;

extern "C" {
    static end: usize;
}

use crate::arch::mm::page_table::set_kernel_page_table;
use crate::arch::KERNEL_LINEAR_SPACE_END;
use crate::mm::paging::kernel_page_table::kernel_page_table;
use rtl::vmm::types::*;
use rtl::vmm::MappingType;

pub fn init_linear_map() {
    let mut tt = kernel_page_table();

    let e = unsafe { *VirtAddr::from_raw(&end as *const _).round_up_page() };

    tt.map_hugepages(
        None,
        MemRange::new(e, KERNEL_LINEAR_SPACE_END - e.bits()),
        MappingType::KERNEL_DATA,
    )
    .expect("Failed to initialize kernel linear map");

    println!(
        "Inited kernel linear map [{:x}; {:x})",
        e.bits(),
        KERNEL_LINEAR_SPACE_END
    );

    unsafe { set_kernel_page_table((*tt).base()) };
}
