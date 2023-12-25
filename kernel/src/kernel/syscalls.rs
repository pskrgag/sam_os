use crate::arch::interrupts::ExceptionCtx;
use crate::kernel::sched::current;
use rtl::syscalls::SyscallList;
use rtl::error::ErrorType;

pub fn do_syscall(ctx: &ExceptionCtx) -> Result<usize, ErrorType> {
    match SyscallList::from_bits(ctx.syscall_number()) {
        Some(SyscallList::SYS_WRITE) => unsafe {
            do_write(core::slice::from_raw_parts(
                ctx.syscall_arg1::<usize>() as *const u8,
                ctx.syscall_arg2(),
            ))
        },
        Some(SyscallList::SYS_VM_ALLOCATE) => {
            let task = current().unwrap().task();
            let vms = task.vms();

            let v = match vms
                .vm_allocate(ctx.syscall_arg1(), ctx.syscall_arg2()) {
                    Ok(v) => v,
                    Err(_) => return Err(ErrorType::INVALID_ARGUMENT),
                };

            Ok(v.into())
        }
        _ => Err(ErrorType::NO_OPERATION),
    }
}

fn do_write(string: &[u8]) -> Result<usize, ErrorType> {
    match core::str::from_utf8(string) {
        Ok(s) => {
            print!("{}", s);
            Ok(0)
        }
        _ => { Err(ErrorType::FAULT) }
    }
}
