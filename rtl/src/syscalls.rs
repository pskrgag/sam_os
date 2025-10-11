#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum SyscallList {
    SYS_WRITE = 0,
    SYS_INVOKE = 1,
    SYS_YIELD = 2,
    SYS_CREATE_TASK = 3,
    SYS_CREATE_PORT = 4,
    SYS_VM_ALLOCATE = 5,
    SYS_CREATE_VMO = 6,
    SYS_VM_FREE = 7,
    SYS_MAP_VMO = 8,
    SYS_MAP_PHYS = 9,
    SYS_TASK_START = 10,
    SYS_TASK_GET_VMS = 11,
    SYS_CLOSE_HANDLE = 12,
    SYS_PORT_CALL = 13,
    SYS_PORT_SEND_WAIT = 14,
    SYS_PORT_RECEIVE = 15,
    SYS_CLONE_HANDLE = 16,
    SYS_MAX = 17,
}

impl TryFrom<usize> for SyscallList {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < Self::SYS_MAX as usize {
            Ok(unsafe { core::mem::transmute(value) })
        } else {
            panic!("")
        }
    }
}

impl From<SyscallList> for usize {
    fn from(value: SyscallList) -> Self {
        value as usize
    }
}
