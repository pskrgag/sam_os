use hal::address::*;

#[repr(C)]
#[derive(Debug)]
struct FpEntry {
    next: *const FpEntry,
    addr: usize,
}

/* SAFETY: fp should be valid and mapped */
/* TODO: This should be ExceptionCtx member function */
pub unsafe fn backtrace(buf: &mut [VirtAddr], fp: VirtAddr) -> usize {
    unsafe {
        let mut fp: *const FpEntry = fp.bits() as *const _;
        let mut num_entries = 0;

        if buf.is_empty() {
            return 0;
        }

        while fp as usize != 0
            && (fp as usize).is_multiple_of(8)
            && !(*fp).next.is_null()
            && (*fp).addr != 0
            && num_entries < buf.len()
        {
            buf[num_entries] = VirtAddr::from((*fp).addr);
            fp = (*fp).next as *const _;
            num_entries += 1;
        }

        num_entries
    }
}
