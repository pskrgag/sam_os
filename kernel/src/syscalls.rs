use adt::vec::Vec;
use crate::drivers::fdt::fdt;
use crate::logger::print_str;
use crate::object::{
    capabilities::{Capability, CapabilityMask},
    factory_object::Factory,
    handle::Handle,
    port_object::Port,
    {wait_many, WaitManyArg},
};
use crate::{
    mm::{
        user_buffer::UserPtr,
        vmm::{vmo::VmObject, vms::Vms},
    },
    sched::current_task,
    tasks::{task::Task, thread::Thread},
};
use alloc::string::String;
use alloc::string::ToString;
use hal::address::*;
use rtl::handle::{HandleBase, HANDLE_INVALID};
use rtl::signal::{Signal, Signals, WaitEntry};
use rtl::vmm::MappingType;
use rtl::{error::ErrorType, ipc::IpcMessage, syscalls::SyscallList};

#[derive(Debug)]
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

    pub fn try_arg<T: TryFrom<usize>>(&self, n: usize) -> Result<T, <T as TryFrom<usize>>::Error> {
        self.args[n].try_into()
    }

    pub fn args(&self) -> [usize; 7] {
        self.args
    }
}

fn read_user_string(source: usize, size: usize) -> Result<String, ErrorType> {
    let name = UserPtr::new_array(source as *const u8, size);
    let name = name.read_on_heap()?;
    let name = core::str::from_utf8(&name)
        .map_err(|_| ErrorType::InvalidArgument)?
        .to_string();

    Ok(name)
}

pub async fn do_syscall(args: SyscallArgs) -> Result<usize, ErrorType> {
    let task = current_task();

    match args.number() {
        SyscallList::Write => {
            let str = UserPtr::new_array(args.arg::<usize>(0) as *const u8, args.arg(1));

            do_write(&str.read_on_heap()?);
            Ok(0)
        }
        SyscallList::CreateTask => {
            let name = read_user_string(args.arg(1), args.arg(2))?;
            let mut table = task.handle_table().await?;

            let factory = table
                .find::<Factory>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;

            Ok(table.add(factory.create_task(name.as_str())?))
        }
        SyscallList::CreatePort => {
            let mut table = task.handle_table().await?;
            let factory = table
                .find::<Factory>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;

            let handle = factory.create_port()?;
            Ok(table.add(handle))
        }
        SyscallList::VmAllocate => {
            let table = task.handle_table().await?;
            let vms = table
                .find::<Vms>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;

            vms.vm_allocate(
                args.arg(1),
                args.try_arg(2).map_err(|_| ErrorType::InvalidArgument)?,
            )
            .await
            .map(|x| x.bits())
        }
        SyscallList::VmFree => {
            let table = task.handle_table().await?;
            let vms = table
                .find::<Vms>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;

            vms.vm_free(args.arg(1), args.arg(2)).await.map(|_| 0)
        }
        SyscallList::CreateVmo => {
            let mut table = task.handle_table().await?;
            let vms = table
                .find::<Vms>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;

            Ok(table.add(vms.create_vmo(
                args.arg(1),
                args.try_arg(2).map_err(|_| ErrorType::InvalidArgument)?,
            )?))
        }
        SyscallList::MapVmo => {
            let table = task.handle_table().await?;
            let vms = table
                .find::<Vms>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;
            let vmo = table
                .find::<VmObject>(args.arg(1), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;
            let to: VirtAddr = args.arg(2);
            let tp: MappingType = args.try_arg(3).map_err(|_| ErrorType::InvalidArgument)?;

            if tp.is_greater(vmo.mapping_type()) {
                return Err(ErrorType::InvalidArgument);
            }

            let range = vmo.range();
            let va_range = if to == VirtAddr::from_bits(0) {
                None
            } else {
                Some(MemRange::new(to, range.size()))
            };

            vms.vm_map(va_range, range, tp).await.map(|x| x.into())
        }
        SyscallList::MapPhys => {
            let table = task.handle_table().await?;
            let vms = table
                .find::<Vms>(args.arg(0), CapabilityMask::from(Capability::MapPhys))
                .ok_or(ErrorType::InvalidHandle)?;

            vms.map_phys(args.arg(1), args.arg(2))
                .await
                .map(|x| x as usize)
        }
        SyscallList::Yield => {
            Thread::self_yield().await;
            Ok(0)
        }
        SyscallList::TaskStart => {
            let table = task.handle_table().await?;
            let task = table
                .find::<Task>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;

            let obj = if args.arg::<HandleBase>(1) != HANDLE_INVALID {
                Some(
                    table
                        .find_raw_handle(args.arg(2))
                        .ok_or(ErrorType::InvalidHandle)?,
                )
            } else {
                None
            };

            task.start(args.arg(1), obj).await.map(|_| 0)
        }
        SyscallList::TaskGetVms => {
            let mut table = task.handle_table().await?;
            let task = table
                .find::<Task>(args.arg(0), CapabilityMask::any())
                .ok_or(ErrorType::InvalidHandle)?;
            let vms = task.vms();

            Ok(table.add(Handle::new(vms.clone(), CapabilityMask::any())))
        }
        SyscallList::PortCall => {
            let port = {
                let table = task.handle_table().await?;

                table
                    .find::<Port>(args.arg(0), CapabilityMask::from(Capability::Call))
                    .ok_or(ErrorType::InvalidHandle)?
            };

            port.call(UserPtr::new(args.arg::<usize>(1) as *mut IpcMessage))
                .await
        }
        SyscallList::PortReply => {
            let msg = UserPtr::new(args.arg::<usize>(2) as *mut IpcMessage);
            let port = {
                let table = task.handle_table().await?;

                table
                    .find::<Port>(args.arg(0), CapabilityMask::from(Capability::Send))
                    .ok_or(ErrorType::InvalidHandle)?
            };

            port.reply(args.arg(1), msg).await.map(|_| 0)
        }
        SyscallList::PortReceive => {
            let in_msg = UserPtr::new(args.arg::<usize>(1) as *mut IpcMessage);
            let port = {
                let table = task.handle_table().await?;

                table
                    .find::<Port>(args.arg(0), CapabilityMask::from(Capability::Receive))
                    .ok_or(ErrorType::InvalidHandle)?
            };

            port.receive(in_msg).await
        }
        SyscallList::PortSend => {
            let in_msg = UserPtr::new(args.arg::<usize>(1) as *mut IpcMessage);
            let port = {
                let table = task.handle_table().await?;

                table
                    .find::<Port>(args.arg(0), CapabilityMask::from(Capability::Receive))
                    .ok_or(ErrorType::InvalidHandle)?
            };

            port.send(in_msg).await.map(|_| 0)
        }
        SyscallList::CloseHandle => {
            let mut table = task.handle_table().await?;

            if table.remove(args.arg(0)) {
                Ok(0)
            } else {
                Err(ErrorType::InvalidHandle)
            }
        }
        SyscallList::CloneHandle => {
            let mut table = task.handle_table().await?;
            let obj = table
                .find_raw_handle(args.arg(0))
                .ok_or(ErrorType::InvalidHandle)?;

            Ok(table.add(obj))
        }
        SyscallList::MapFdt => {
            let fdt_pa: PhysAddr = fdt().base.into();
            let fdt_size = fdt().size;

            task.vms()
                .map_phys(fdt_pa, fdt_size)
                .await
                .map(|x| x as usize)
        }
        SyscallList::WaitObject => {
            let sig: Signals = args.try_arg(1)?;
            let obj = {
                let table = task.handle_table().await?;

                table
                    .find_poly(args.arg(0), CapabilityMask::from(Capability::Wait))
                    .ok_or(ErrorType::InvalidHandle)?
            };

            obj.wait_signal(sig).await.map(|_| 0)
        }
        SyscallList::WaitObjectMany => {
            let mut user_ptr =
                UserPtr::new_array(args.arg::<usize>(0) as *mut WaitEntry, args.arg(1));
            let mut user_wait_entries = user_ptr.read_on_heap()?;
            let mut wait_entries = Vec::new();

            {
                let table = task.handle_table().await?;

                for e in user_wait_entries.iter().map(|x| {
                    Ok(WaitManyArg {
                        obj: table
                            .find_poly(x.handle, CapabilityMask::from(Capability::Wait))
                            .ok_or(ErrorType::InvalidHandle)?,
                        waitfor: x.waitfor,
                        pending: Signal::None.into(),
                    })
                }) {
                    wait_entries.try_push(e?)?;
                }
            }

            wait_many(&mut wait_entries).await?;

            for (user, kernel) in core::iter::zip(user_wait_entries.iter_mut(), wait_entries.iter())
            {
                user.pendind = kernel.pending;
            }

            user_ptr.write_array(&user_wait_entries)?;
            Ok(0)
        }
    }
}

fn do_write(string: &[u8]) {
    let str = unsafe { core::str::from_utf8_unchecked(string) };
    // let str = alloc::format!("{} --> {str}\n", current_task().name());
    print_str(str);
}
