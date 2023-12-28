use crate::kernel::object::handle::Handle;
use crate::kernel::sched::current;
use crate::kernel::object::task_object::Task;
use crate::kernel::object::thread_object::Thread;
use alloc::string::ToString;
use alloc::vec::Vec;
use rtl::error::ErrorType;
use rtl::handle::HandleBase;
use rtl::syscalls::SyscallList;
use rtl::vmm::types::VirtAddr;

pub struct SyscallArgs {
    number: SyscallList,
    args: [usize; 7],
}

impl SyscallArgs {
    pub fn new(number: usize, args: [usize; 7]) -> Option<Self> {
        Some(Self {
            number: SyscallList::from_bits(number)?,
            args,
        })
    }

    pub fn number(&self) -> SyscallList {
        self.number
    }

    pub fn arg<T: From<usize>>(&self, n: usize) -> T {
        self.args[n].into()
    }

    pub fn args(&self) -> [usize; 7] {
        self.args
    }
}

pub fn do_syscall(args: SyscallArgs) -> Result<usize, ErrorType> {
    match args.number() {
        SyscallList::SYS_WRITE => unsafe {
            do_write(core::slice::from_raw_parts(
                args.arg::<usize>(0) as *const u8,
                args.arg(1),
            ))
        },
        SyscallList::SYS_TASK_CREATE_FROM_VMO => {
            use crate::kernel::object::vm_object::VmObject;

            let name_range = unsafe {
                core::slice::from_raw_parts(
                    args.arg::<usize>(0) as *const u8,
                    args.arg(1),
                )
            };
            let name = core::str::from_utf8(name_range).map_err(|_| ErrorType::FAULT)?;
            let handles = unsafe {
                core::slice::from_raw_parts(
                    args.arg::<usize>(2) as *const HandleBase,
                    args.arg(3),
                )
            };
            let ep = args.arg::<VirtAddr>(4);

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
            new_task.add_initial_thread(init_thread);

            let handle = Handle::new::<Task>(new_task.clone());
            let ret = handle.as_raw();

            table.add(handle);

            Ok(ret)
        }
        SyscallList::SYS_INVOKE => {
            let task = current().unwrap().task();
            let table = task.handle_table();

            let req_t = table
                .find_poly(args.arg(0))
                .ok_or(ErrorType::INVALID_ARGUMENT)?;

            // Drop locks
            drop(table);
            drop(task);

            req_t.invoke(&args.args()[1..])
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
