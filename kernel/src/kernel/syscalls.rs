use crate::{
    kernel::{
        object::{
            factory_object::Factory,
            port_object::Port,
            task_object::Task,
            vm_object::VmObject,
            vms_object::{VmoCreateArgs, Vms},
        },
        sched::current,
    },
    mm::user_buffer::UserPtr,
};
use alloc::string::String;
use alloc::string::ToString;
use rtl::handle::{HandleBase, HANDLE_INVALID};
use rtl::vmm::types::Address;
use rtl::{error::ErrorType, ipc::IpcMessage, syscalls::SyscallList};

pub struct SyscallArgs {
    number: SyscallList,
    args: [usize; 7],
}

impl SyscallArgs {
    pub fn new(number: usize, args: [usize; 7]) -> Option<Self> {
        Some(Self {
            number: SyscallList::try_from(number).unwrap(),
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

fn read_user_string(source: usize, size: usize) -> Result<String, ErrorType> {
    let name = UserPtr::new_array(source as *const u8, size);
    let name = name.read_on_heap().ok_or(ErrorType::Fault)?;
    let name = core::str::from_utf8(&name)
        .map_err(|_| ErrorType::InvalidArgument)?
        .to_string();

    Ok(name)
}

pub fn do_syscall(args: SyscallArgs) -> Result<usize, ErrorType> {
    let thread = current().unwrap();
    let task = thread.task();
    let mut table = task.handle_table();

    match args.number() {
        SyscallList::Write => unsafe {
            do_write(core::slice::from_raw_parts(
                args.arg::<usize>(0) as *const u8,
                args.arg(1),
            ))
        },
        SyscallList::CreateTask => {
            let name = read_user_string(args.arg(1), args.arg(2))?;

            let factory = table
                .find::<Factory>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            Ok(table.add(factory.create_task(name.as_str())?))
        }
        SyscallList::CreatePort => {
            let factory = table
                .find::<Factory>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            let handle = factory.create_port()?;
            Ok(table.add(handle))
        }
        SyscallList::VmAllocate => {
            let vms = table
                .find::<Vms>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            vms.vm_allocate(args.arg(1), args.arg(2))
                .map(|x| x.bits())
                .map_err(|_| ErrorType::NoMemory)
        }
        SyscallList::VmFree => {
            let vms = table
                .find::<Vms>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            vms.vm_free(args.arg(1), args.arg(2)).map(|_| 0)
        }
        SyscallList::CreateVmo => {
            use rtl::objects::vmo::VmoFlags;

            let vms = table
                .find::<Vms>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            let flags = VmoFlags::from_bits(args.arg(5)).ok_or(ErrorType::InvalidArgument)?;
            let vmo_args = match flags {
                VmoFlags::BACKED => VmoCreateArgs::Backed(
                    UserPtr::new_array(args.arg::<usize>(1) as *const u8, args.arg(2)),
                    args.arg(3),
                    args.arg(4),
                ),
                VmoFlags::ZEROED => VmoCreateArgs::Zeroed(args.arg(2), args.arg(3), args.arg(4)),
                _ => Err(ErrorType::InvalidArgument)?,
            };

            Ok(table.add(vms.create_vmo(vmo_args)?))
        }
        SyscallList::MapVmo => {
            let vms = table
                .find::<Vms>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;
            let vmo = table
                .find::<VmObject>(args.arg(1))
                .ok_or(ErrorType::InvalidHandle)?;

            let ranges = vmo.as_ranges();

            vms.vm_map(ranges.0, ranges.1, vmo.mapping_type())
                .map(|x| x.into())
                .map_err(|_| ErrorType::InvalidArgument)
        }
        SyscallList::MapPhys => {
            let vms = table
                .find::<Vms>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            vms.map_phys(args.arg(1), args.arg(2)).map(|x| x as usize)
        }
        SyscallList::Yield => {
            let thread = current().unwrap();
            thread.self_yield();

            Ok(0)
        }
        SyscallList::TaskStart => {
            let task = table
                .find::<Task>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            let obj = if args.arg::<HandleBase>(1) != HANDLE_INVALID {
                Some(
                    table
                        .find_poly(args.arg(2))
                        .ok_or(ErrorType::InvalidHandle)?,
                )
            } else {
                None
            };

            task.start(args.arg(1), obj).map(|_| 0)
        }
        SyscallList::TaskGetVms => {
            let task = table
                .find::<Task>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;
            let vms = task.vms();

            Ok(table.add(vms))
        }
        SyscallList::PortCall => {
            let port = table
                .find::<Port>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            drop(table);
            port.call(UserPtr::new(args.arg::<usize>(1) as *mut IpcMessage))
                .map(|_| 0)
        }
        SyscallList::PortSendWait => {
            let port = table
                .find::<Port>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;
            let msg = UserPtr::new(args.arg::<usize>(2) as *mut IpcMessage);

            drop(table);
            port.send_wait(args.arg(1), msg)
        }
        SyscallList::PortReceive => {
            let port = table
                .find::<Port>(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;
            let in_msg = UserPtr::new(args.arg::<usize>(1) as *mut IpcMessage);

            drop(table);
            port.receive(in_msg)
        }
        SyscallList::CloseHandle => {
            if table.remove(args.arg(0)) {
                Ok(0)
            } else {
                Err(ErrorType::InvalidHandle)
            }
        }
        SyscallList::CloneHandle => {
            let obj = table
                .find_poly(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            Ok(table.add(obj))
        }
        _ => Err(ErrorType::NoOperation),
    }
}

fn do_write(string: &[u8]) -> Result<usize, ErrorType> {
    match core::str::from_utf8(string) {
        Ok(s) => {
            print!("{}", s);
            Ok(0)
        }
        _ => Err(ErrorType::Fault),
    }
}
