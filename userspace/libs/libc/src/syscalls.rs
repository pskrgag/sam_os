#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

#[cfg(target_arch = "x86_64")]
use crate::syscalls_x86_64::*;

use super::handle::Handle;
use hal::address::{Address, PhysAddr, VirtAddr};
use rtl::error::ErrorType;
use rtl::handle::Handle as RawHandle;
use rtl::ipc::IpcMessage;
use rtl::signal::{Signals, WaitEntry};
use rtl::syscalls::SyscallList;
use rtl::vmm::MappingType;

pub enum Syscall<'a> {
    Write(&'a str),
    Yield,
    CreatePort(RawHandle),
    CreateTask(RawHandle, &'a str),
    VmAllocate(RawHandle, usize, MappingType),
    VmFree(RawHandle, *mut u8, usize),
    VmCreateVmo(RawHandle, usize, MappingType),
    VmMapVmo(RawHandle, RawHandle, VirtAddr, MappingType),
    VmMapPhys(RawHandle, PhysAddr, usize),
    TaskStart(RawHandle, VirtAddr, RawHandle),
    VmsHandle(RawHandle),
    CloseHandle(RawHandle),
    PortCall(RawHandle, *mut IpcMessage<'a>),
    PortSend(RawHandle, *mut IpcMessage<'a>),
    PortReplyWait(RawHandle, RawHandle, *mut IpcMessage<'a>),
    PortReply(RawHandle, RawHandle, *const IpcMessage<'a>),
    PortReceive(RawHandle, *mut IpcMessage<'a>),
    CloneHandle(RawHandle),
    GetFdt,
    ObjectWait(RawHandle, Signals),
    ObjectWaitMany(&'a mut [WaitEntry]),
}

impl<'a> Syscall<'a> {
    pub fn debug_write(s: &'a str) -> Result<(), ErrorType> {
        unsafe { syscall(Self::Write(s).as_args())? };
        Ok(())
    }

    pub fn sys_yield() {
        unsafe { syscall(Self::Yield.as_args()).unwrap() };
    }

    pub fn create_port(h: &Handle) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::CreatePort(h.as_raw()).as_args()).map(Handle::new) }
    }

    pub fn create_task(h: &Handle, name: &'a str) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::CreateTask(h.as_raw(), name).as_args()).map(Handle::new) }
    }

    pub fn vm_allocate(h: &Handle, size: usize, mt: MappingType) -> Result<*mut u8, ErrorType> {
        unsafe { syscall(Self::VmAllocate(h.as_raw(), size, mt).as_args()).map(|x| x as *mut u8) }
    }

    pub fn vm_free(h: &Handle, ptr: *mut u8, size: usize) -> Result<(), ErrorType> {
        unsafe { syscall(Self::VmFree(h.as_raw(), ptr, size).as_args()).map(|_| ()) }
    }

    pub fn vm_create_vmo(h: &Handle, size: usize, tp: MappingType) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::VmCreateVmo(h.as_raw(), size, tp).as_args()).map(Handle::new) }
    }

    pub fn vm_map_vmo(
        vms: &Handle,
        vmo: &Handle,
        to: VirtAddr,
        tp: MappingType,
    ) -> Result<VirtAddr, ErrorType> {
        unsafe {
            syscall(Self::VmMapVmo(vms.as_raw(), vmo.as_raw(), to, tp).as_args())
                .map(<VirtAddr as Address>::from_bits)
        }
    }

    pub fn vm_map_phys(vms: &Handle, pa: PhysAddr, size: usize) -> Result<*mut u8, ErrorType> {
        unsafe { syscall(Self::VmMapPhys(vms.as_raw(), pa, size).as_args()).map(|x| x as _) }
    }

    pub fn task_start(task: &Handle, ep: VirtAddr, boot_handle: &Handle) -> Result<(), ErrorType> {
        unsafe {
            syscall(Self::TaskStart(task.as_raw(), ep, boot_handle.as_raw()).as_args()).map(|_| ())
        }
    }

    pub fn task_get_vms(h: &Handle) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::VmsHandle(h.as_raw()).as_args()).map(Handle::new) }
    }

    pub fn close_handle(h: RawHandle) -> Result<(), ErrorType> {
        unsafe { syscall(Self::CloseHandle(h).as_args()).map(|_| ()) }
    }

    pub fn port_call(h: &Handle, msg: *mut IpcMessage<'a>) -> Result<usize, ErrorType> {
        unsafe { syscall(Self::PortCall(h.as_raw(), msg).as_args()) }
    }

    pub fn port_send(h: &Handle, msg: *mut IpcMessage<'a>) -> Result<(), ErrorType> {
        unsafe { syscall(Self::PortSend(h.as_raw(), msg).as_args()).map(|_| ()) }
    }

    pub fn port_receive(h: &Handle, msg: *mut IpcMessage<'a>) -> Result<usize, ErrorType> {
        unsafe { syscall(Self::PortReceive(h.as_raw(), msg).as_args()) }
    }

    pub fn clone_handle(h: &Handle) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::CloneHandle(h.as_raw()).as_args()).map(Handle::new) }
    }

    pub fn get_fdt() -> Result<VirtAddr, ErrorType> {
        unsafe { syscall(Self::GetFdt.as_args()).map(<VirtAddr as Address>::from_bits) }
    }

    pub fn object_wait(h: &Handle, sig: Signals) -> Result<(), ErrorType> {
        unsafe { syscall(Self::ObjectWait(h.as_raw(), sig).as_args()).map(|_| ()) }
    }

    pub fn object_wait_many(wait_entries: &'a mut [WaitEntry]) -> Result<(), ErrorType> {
        unsafe { syscall(Self::ObjectWaitMany(wait_entries).as_args()).map(|_| ()) }
    }

    pub fn port_send_wait(
        h: &Handle,
        reply_port: Handle,
        msg: *mut IpcMessage<'a>,
    ) -> Result<usize, ErrorType> {
        unsafe {
            syscall(
                Self::PortReplyWait(
                    h.as_raw(),
                    {
                        let raw = reply_port.as_raw();

                        core::mem::forget(reply_port);
                        raw
                    },
                    msg,
                )
                .as_args(),
            )
        }
    }

    pub fn port_reply(
        h: &Handle,
        reply_port: Handle,
        msg: *const IpcMessage<'a>,
    ) -> Result<(), ErrorType> {
        unsafe {
            syscall(
                Self::PortReply(
                    h.as_raw(),
                    {
                        let raw = reply_port.as_raw();

                        core::mem::forget(reply_port);
                        raw
                    },
                    msg,
                )
                .as_args(),
            )
            .map(|_| ())
        }
    }

    pub fn as_args(self) -> [usize; 8] {
        match self {
            Syscall::Write(string) => [
                SyscallList::Write.into(),
                string.as_ptr() as usize,
                string.len(),
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::CreatePort(handle) => {
                [SyscallList::CreatePort.into(), handle, 0, 0, 0, 0, 0, 0]
            }
            Syscall::VmAllocate(handle, size, tp) => [
                SyscallList::VmAllocate.into(),
                handle,
                size,
                tp as usize,
                0,
                0,
                0,
                0,
            ],
            Syscall::CreateTask(handle, name) => [
                SyscallList::CreateTask.into(),
                handle,
                name.as_ptr() as usize,
                name.len(),
                0,
                0,
                0,
                0,
            ],
            Syscall::VmFree(handle, ptr, size) => [
                SyscallList::VmFree.into(),
                handle,
                ptr as usize,
                size,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmCreateVmo(handle, size, tp) => [
                SyscallList::CreateVmo.into(),
                handle,
                size,
                tp as usize,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmMapVmo(vms, vmo, to, tp) => [
                SyscallList::MapVmo.into(),
                vms,
                vmo,
                to.bits(),
                tp as usize,
                0,
                0,
                0,
            ],
            Syscall::TaskStart(handle, ep, boot_handle) => [
                SyscallList::TaskStart.into(),
                handle,
                ep.into(),
                boot_handle,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmMapPhys(vms, pa, size) => [
                SyscallList::MapPhys.into(),
                vms,
                pa.into(),
                size,
                0,
                0,
                0,
                0,
            ],
            Syscall::CloseHandle(handle) => {
                [SyscallList::CloseHandle.into(), handle, 0, 0, 0, 0, 0, 0]
            }
            Syscall::PortCall(handle, msg) => [
                SyscallList::PortCall.into(),
                handle,
                msg as *mut _ as usize,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::PortSend(handle, msg) => [
                SyscallList::PortSend.into(),
                handle,
                msg as *mut _ as usize,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::PortReceive(handle, msg) => [
                SyscallList::PortReceive.into(),
                handle,
                msg as *mut _ as usize,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::PortReplyWait(handle, reply_port, msg) => [
                SyscallList::PortReplyWait.into(),
                handle,
                reply_port,
                msg as *const _ as usize,
                0,
                0,
                0,
                0,
            ],
            Syscall::PortReply(handle, reply_port, msg) => [
                SyscallList::PortReply.into(),
                handle,
                reply_port,
                msg as *const _ as usize,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmsHandle(h) => [SyscallList::TaskGetVms.into(), h, 0, 0, 0, 0, 0, 0],
            Syscall::Yield => [SyscallList::Yield.into(), 0, 0, 0, 0, 0, 0, 0],
            Syscall::CloneHandle(h) => [SyscallList::CloneHandle.into(), h, 0, 0, 0, 0, 0, 0],
            Syscall::GetFdt => [SyscallList::MapFdt.into(), 0, 0, 0, 0, 0, 0, 0],
            Syscall::ObjectWait(h, sig) => [
                SyscallList::WaitObject.into(),
                h,
                (*sig).into(),
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::ObjectWaitMany(entries) => [
                SyscallList::WaitObjectMany.into(),
                entries.as_mut_ptr() as usize,
                entries.len(),
                0,
                0,
                0,
                0,
                0,
            ],
        }
    }
}
