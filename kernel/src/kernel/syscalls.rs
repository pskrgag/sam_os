use crate::arch::interrupts::ExceptionCtx;
use crate::kernel::sched::current;
use bitflags::bitflags;

bitflags! {
    struct SyscallList: usize {
        const SYS_WRITE = 0;
        const SYS_VM_ALLOCATE = 1;
    }
}

pub fn do_syscall(ctx: &ExceptionCtx) -> usize {
    match SyscallList::from_bits(ctx.syscall_number()) {
        Some(SyscallList::SYS_WRITE) => unsafe {
            do_write(core::slice::from_raw_parts(
                ctx.syscall_arg1() as *const u8,
                ctx.syscall_arg2(),
            ))
        },
        Some(SyscallList::SYS_VM_ALLOCATE) => {
            let task = current().unwrap().task();
            let vms = task.vms();

            let v = vms
                .vm_allocate(0x1000, crate::mm::paging::page_table::MappingType::UserData)
                .unwrap();

            v.into()
        }
        _ => usize::MAX,
    }
}

fn do_write(string: &[u8]) -> usize {
    match core::str::from_utf8(string) {
        Ok(s) => {
            print!("{}", s);
            0
        }
        _ => usize::MAX,
    }
}
