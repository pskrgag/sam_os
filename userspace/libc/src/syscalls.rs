#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

#[cfg(target_arch = "x86_64")]
use crate::syscalls_x86_64::*;

use super::handle::Handle;
use rtl::error::ErrorType;
use rtl::handle::Handle as RawHandle;
use rtl::ipc::IpcMessage;
use rtl::objects::vmo::VmoFlags;
use rtl::syscalls::SyscallList;
use rtl::vmm::{
    types::{PhysAddr, VirtAddr},
    MappingType,
};

pub enum VmoCreateArgs {
    Backed(*const u8, usize, MappingType, VirtAddr),
    Zeroed(usize, MappingType, VirtAddr),
}

pub enum Syscall<'a> {
    Write(&'a str),
    Yield,
    CreatePort(RawHandle),
    CreateTask(RawHandle, &'a str),
    VmAllocate(RawHandle, usize, MappingType),
    VmFree(RawHandle, *mut u8, usize),
    VmCreateVmo(RawHandle, VmoCreateArgs),
    VmMapVmo(RawHandle, RawHandle),
    VmMapPhys(RawHandle, PhysAddr, usize),
    TaskStart(RawHandle, VirtAddr, RawHandle),
    VmsHandle(RawHandle),
    CloseHandle(RawHandle),
    PortCall(RawHandle, *mut IpcMessage<'a>),
    PortSendWait(RawHandle, RawHandle, *mut IpcMessage<'a>),
    PortReceive(RawHandle, *mut IpcMessage<'a>),
    CloneHandle(RawHandle),
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

    pub fn vm_create_vmo(h: &Handle, args: VmoCreateArgs) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::VmCreateVmo(h.as_raw(), args).as_args()).map(Handle::new) }
    }

    pub fn vm_map_vmo(vms: &Handle, vmo: &Handle) -> Result<(), ErrorType> {
        unsafe { syscall(Self::VmMapVmo(vms.as_raw(), vmo.as_raw()).as_args()).map(|_| ()) }
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

    pub fn port_call(h: &Handle, msg: *mut IpcMessage<'a>) -> Result<(), ErrorType> {
        unsafe { syscall(Self::PortCall(h.as_raw(), msg).as_args()).map(|_| ()) }
    }

    pub fn port_receive(h: &Handle, msg: *mut IpcMessage<'a>) -> Result<usize, ErrorType> {
        unsafe { syscall(Self::PortReceive(h.as_raw(), msg).as_args()) }
    }

    pub fn clone_handle(h: &Handle) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::CloneHandle(h.as_raw()).as_args()).map(Handle::new) }
    }

    pub fn port_send_wait(
        h: &Handle,
        reply_port: Handle,
        msg: *mut IpcMessage<'a>,
    ) -> Result<usize, ErrorType> {
        unsafe {
            syscall(
                Self::PortSendWait(
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
            Syscall::VmCreateVmo(handle, args) => match args {
                VmoCreateArgs::Backed(addr, size, mt, load_addr) => [
                    SyscallList::CreateVmo.into(),
                    handle,
                    addr as usize,
                    size,
                    mt as usize,
                    load_addr.into(),
                    VmoFlags::Backed as usize,
                    0,
                ],
                VmoCreateArgs::Zeroed(size, mt, load_addr) => [
                    SyscallList::CreateVmo.into(),
                    handle,
                    0,
                    size,
                    mt as usize,
                    load_addr.into(),
                    VmoFlags::Zeroed as usize,
                    0,
                ],
            },
            Syscall::VmMapVmo(vms, vmo) => [SyscallList::MapVmo.into(), vms, vmo, 0, 0, 0, 0, 0],
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
            Syscall::PortSendWait(handle, reply_port, msg) => [
                SyscallList::PortSendWait.into(),
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
        }
    }
}
