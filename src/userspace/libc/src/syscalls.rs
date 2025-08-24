#[cfg(target_arch = "aarch64")]
use crate::syscalls_aarch64::*;

#[cfg(target_arch = "x86_64")]
use crate::syscalls_x86_64::*;

use rtl::error::ErrorType;
use rtl::handle::Handle;
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
    CreatePort(Handle),
    CreateTask(Handle, &'a str),
    VmAllocate(Handle, usize, MappingType),
    VmFree(Handle, *mut u8, usize),
    VmCreateVmo(Handle, VmoCreateArgs),
    VmMapVmo(Handle, Handle),
    VmMapPhys(Handle, PhysAddr, usize),
    TaskStart(Handle, VirtAddr, Handle),
    VmsHandle(Handle),
    CloseHandle(Handle),
    PortCall(Handle, *mut IpcMessage<'a>),
    PortSendWait(Handle, Handle, &'a IpcMessage<'a>, &'a mut IpcMessage<'a>),
    PortReceive(Handle, *mut IpcMessage<'a>),
}

impl<'a> Syscall<'a> {
    pub fn debug_write(s: &'a str) -> Result<(), ErrorType> {
        unsafe { syscall(Self::Write(s).as_args())? };
        Ok(())
    }

    pub fn sys_yield() {
        unsafe { syscall(Self::Yield.as_args()).unwrap() };
    }

    pub fn create_port(h: Handle) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::CreatePort(h).as_args()) }
    }

    pub fn create_task(h: Handle, name: &'a str) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::CreateTask(h, name).as_args()) }
    }

    pub fn vm_allocate(h: Handle, size: usize, mt: MappingType) -> Result<*mut u8, ErrorType> {
        unsafe { syscall(Self::VmAllocate(h, size, mt).as_args()).map(|x| x as *mut u8) }
    }

    pub fn vm_free(h: Handle, ptr: *mut u8, size: usize) -> Result<(), ErrorType> {
        unsafe { syscall(Self::VmFree(h, ptr, size).as_args()).map(|_| ()) }
    }

    pub fn vm_create_vmo(h: Handle, args: VmoCreateArgs) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::VmCreateVmo(h, args).as_args()) }
    }

    pub fn vm_map_vmo(vms: Handle, vmo: Handle) -> Result<(), ErrorType> {
        unsafe { syscall(Self::VmMapVmo(vms, vmo).as_args()).map(|_| ()) }
    }

    pub fn vm_map_phys(vms: Handle, pa: PhysAddr, size: usize) -> Result<*mut u8, ErrorType> {
        unsafe { syscall(Self::VmMapPhys(vms, pa, size).as_args()).map(|x| x as _) }
    }

    pub fn task_start(task: Handle, ep: VirtAddr, boot_handle: Handle) -> Result<(), ErrorType> {
        unsafe { syscall(Self::TaskStart(task, ep, boot_handle).as_args()).map(|_| ()) }
    }

    pub fn task_get_vms(h: Handle) -> Result<Handle, ErrorType> {
        unsafe { syscall(Self::VmsHandle(h).as_args()) }
    }

    pub fn close_handle(h: Handle) -> Result<(), ErrorType> {
        unsafe { syscall(Self::CloseHandle(h).as_args()).map(|_| ()) }
    }

    pub fn port_call(h: Handle, msg: *mut IpcMessage<'a>) -> Result<(), ErrorType> {
        unsafe { syscall(Self::PortCall(h, msg).as_args()).map(|_| ()) }
    }

    pub fn port_receive(h: Handle, msg: *mut IpcMessage<'a>) -> Result<usize, ErrorType> {
        unsafe { syscall(Self::PortReceive(h, msg).as_args()) }
    }

    pub fn port_send_wait(
        h: Handle,
        reply_port: Handle,
        in_msg: &'a IpcMessage<'a>,
        out_msg: &'a mut IpcMessage<'a>,
    ) -> Result<(), ErrorType> {
        unsafe { syscall(Self::PortSendWait(h, reply_port, in_msg, out_msg).as_args()).map(|_| ()) }
    }

    pub fn as_args(self) -> [usize; 8] {
        match self {
            Syscall::Write(string) => [
                SyscallList::SYS_WRITE.into(),
                string.as_ptr() as usize,
                string.len(),
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::CreatePort(handle) => [
                SyscallList::SYS_CREATE_PORT.into(),
                handle,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmAllocate(handle, size, tp) => [
                SyscallList::SYS_VM_ALLOCATE.into(),
                handle,
                size,
                tp.into(),
                0,
                0,
                0,
                0,
            ],
            Syscall::CreateTask(handle, name) => [
                SyscallList::SYS_CREATE_TASK.into(),
                handle,
                name.as_ptr() as usize,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmFree(handle, ptr, size) => [
                SyscallList::SYS_VM_FREE.into(),
                handle,
                ptr as usize,
                size,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmCreateVmo(handle, args) => match args {
                VmoCreateArgs::Backed(adr, size, mt, load_addr) => [
                    SyscallList::SYS_CREATE_VMO.into(),
                    handle,
                    adr as usize,
                    size,
                    mt.into(),
                    load_addr.into(),
                    VmoFlags::BACKED.bits(),
                    0,
                ],
                VmoCreateArgs::Zeroed(size, mt, load_addr) => [
                    SyscallList::SYS_CREATE_VMO.into(),
                    handle,
                    0,
                    size,
                    mt.bits(),
                    load_addr.into(),
                    VmoFlags::ZEROED.bits(),
                    0,
                ],
            },
            Syscall::VmMapVmo(vms, vmo) => {
                [SyscallList::SYS_MAP_VMO.into(), vms, vmo, 0, 0, 0, 0, 0]
            }
            Syscall::TaskStart(handle, ep, boot_handle) => [
                SyscallList::SYS_TASK_START.into(),
                handle,
                ep.into(),
                boot_handle,
                0,
                0,
                0,
                0,
            ],
            Syscall::VmMapPhys(vms, pa, size) => [
                SyscallList::SYS_MAP_PHYS.into(),
                vms,
                pa.into(),
                size,
                0,
                0,
                0,
                0,
            ],
            Syscall::CloseHandle(handle) => [
                SyscallList::SYS_CLOSE_HANDLE.into(),
                handle,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::PortCall(handle, msg) => [
                SyscallList::SYS_PORT_CALL.into(),
                handle,
                msg as *mut _ as usize,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::PortReceive(handle, msg) => [
                SyscallList::SYS_PORT_RECEIVE.into(),
                handle,
                msg as *mut _ as usize,
                0,
                0,
                0,
                0,
                0,
            ],
            Syscall::PortSendWait(handle, reply_port, in_msg, out_msg) => [
                SyscallList::SYS_PORT_SEND_WAIT.into(),
                handle,
                reply_port,
                in_msg as *const _ as usize,
                out_msg as *mut _ as usize,
                0,
                0,
                0,
            ],
            Syscall::VmsHandle(h) => [SyscallList::SYS_TASK_GET_VMS.into(), h, 0, 0, 0, 0, 0, 0],
            Syscall::Yield => [SyscallList::SYS_YIELD.into(), 0, 0, 0, 0, 0, 0, 0],
        }
    }
}
