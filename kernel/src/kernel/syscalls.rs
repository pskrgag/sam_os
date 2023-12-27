use crate::arch::interrupts::ExceptionCtx;
use crate::kernel::object::handle::Handle;
use crate::kernel::sched::current;
use crate::kernel::tasks::task::Task;
use crate::kernel::tasks::thread::Thread;
use alloc::string::ToString;
use alloc::vec::Vec;
use rtl::error::ErrorType;
use rtl::handle::HandleBase;
use rtl::syscalls::SyscallList;
use rtl::vmm::types::VirtAddr;

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

            let v = match vms.vm_allocate(ctx.syscall_arg1(), ctx.syscall_arg2()) {
                Ok(v) => v,
                Err(_) => return Err(ErrorType::INVALID_ARGUMENT),
            };

            Ok(v.into())
        }
        Some(SyscallList::SYS_VM_CREATE_VM_OBJECT) => {
            use crate::kernel::object::vm_object::VmObject;

            let range = unsafe {
                core::slice::from_raw_parts(
                    ctx.syscall_arg1::<usize>() as *const u8,
                    ctx.syscall_arg2(),
                )
            };

            let vmo = VmObject::from_buffer(range, ctx.syscall_arg3(), ctx.syscall_arg4())
                .ok_or(ErrorType::NO_MEMORY)?;
            let handle = Handle::new::<VmObject>(vmo.clone());
            let ret = handle.as_raw();

            let task = current().unwrap().task();
            let mut table = task.handle_table();

            table.add(handle);

            Ok(ret)
        }
        Some(SyscallList::SYS_TASK_CREATE_FROM_VMO) => {
            use crate::kernel::object::vm_object::VmObject;

            let name_range = unsafe {
                core::slice::from_raw_parts(
                    ctx.syscall_arg1::<usize>() as *const u8,
                    ctx.syscall_arg2(),
                )
            };
            let name = core::str::from_utf8(name_range).map_err(|_| ErrorType::FAULT)?;
            let handles = unsafe {
                core::slice::from_raw_parts(
                    ctx.syscall_arg3::<usize>() as *const HandleBase,
                    ctx.syscall_arg4(),
                )
            };
            let ep = ctx.syscall_arg5::<VirtAddr>();

            let task = current().unwrap().task();
            let mut table = task.handle_table();

            let handles: Vec<_> = handles
                .iter()
                .map(|x| {
                    table
                        .find::<VmObject>(*x)
                        .ok_or(ErrorType::NO_OPERATION)
                        .unwrap()
                })
                .collect();

            let new_task = Task::new(name.to_string());
            let init_thread = Thread::new(new_task.clone(), 10);
            let vms = new_task.vms();

            for i in handles {
                let r = i.as_ranges();
                vms.vm_map(r.0, r.1, i.mapping_type()).unwrap();
            }

            init_thread.init_user(ep);
            new_task.add_thread(init_thread);

            let handle = Handle::new::<Task>(new_task.clone());
            let ret = handle.as_raw();

            table.add(handle);

            Ok(ret)
        }
        Some(SyscallList::SYS_TASK_START) => {
            let task = current().unwrap().task();
            let table = task.handle_table();

            let req_t = table
                .find::<Task>(ctx.syscall_arg1())
                .ok_or(ErrorType::INVALID_ARGUMENT)?;

            req_t.start();
            Ok(ErrorType::OK.into())
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
        _ => Err(ErrorType::FAULT),
    }
}
