use bitflags::bitflags;

bitflags! {
    struct SyscallList: usize {
        const SYS_WRITE = 0;
    }
}

pub fn do_syscall(
    number: usize,
    x1: usize,
    x2: usize,
    _x3: usize,
    _x4: usize,
    _x5: usize,
) -> usize {
    match SyscallList::from_bits(number) {
        Some(SyscallList::SYS_WRITE) => unsafe {
            do_write(core::slice::from_raw_parts(x1 as *const u8, x2))
        },
        _ => usize::MAX,
    }
}

fn do_write(string: &[u8]) -> usize {
    match core::str::from_utf8(string) {
        Ok(s) => {
            println!("{}", s);
            0
        }
        _ => usize::MAX,
    }
}
