use crate::kernel::sched::current;
use rtl::error::ErrorType;
use rtl::syscalls::SyscallList;

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
        SyscallList::SYS_INVOKE => {
            let task = current().unwrap().task();
            let mut table = task.handle_table();

            if args.arg::<usize>(1) == rtl::handle::HANDLE_CLOSE {
                table.remove(args.arg(0));
                return Ok(0);
            }

            let req_t = table
                .find_poly(args.arg(0))
                .ok_or(ErrorType::INVALID_HANDLE)?;

            // Drop locks
            drop(table);
            drop(task);

            req_t.invoke(&args.args()[1..])
        },
        SyscallList::SYS_YIELD => {
            let thread = current().unwrap();
            thread.self_yield();

            Ok(0)
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
