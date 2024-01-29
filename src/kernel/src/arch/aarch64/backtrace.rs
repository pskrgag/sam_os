use rtl::vmm::types::*;

#[repr(C)]
struct FpEntry {
    next: usize,
    addr: usize,
}

/* SAFETY: fp should be valid and mapped */
/* TODO: This should be ExceptionCtx member function */
pub unsafe fn backtrace(buf: &mut [VirtAddr], fp: VirtAddr) -> usize {
    let mut fp: *const FpEntry = fp.bits() as *const _;
    let mut num_entries = 0;

    if buf.len() == 0 {
        return 0;
    }

    while fp as usize != 0 && (*fp).next != 0 && (*fp).addr != 0 && num_entries < buf.len() {
        buf[num_entries] = VirtAddr::from((*fp).addr);
        fp = (*fp).next as *const _;
        num_entries += 1;
    }

    num_entries
}
