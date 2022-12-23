use crate::mm::types::VirtAddr;
use core::arch::asm;

#[repr(C)]
struct FpEntry {
    next: usize,
    addr: usize,
}

/* SAFETY: fp should be valid and mapped */
pub unsafe fn backtrace(buf: &mut [VirtAddr]) -> usize {
    let mut fp: *const FpEntry;
    let mut num_entries = 0;

    if buf.len() == 0 {
        return 0;
    }

    asm!("mov {}, fp", out(reg) fp);

    while (*fp).next != 0 && (*fp).addr != 0 && num_entries < buf.len() {
        buf[num_entries] = VirtAddr::from((*fp).addr);
        fp = (*fp).next as *const _;
        num_entries += 1;
    }

    num_entries
}
