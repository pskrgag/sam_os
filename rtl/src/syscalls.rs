#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum SyscallList {
    Write = 0,
    Yield = 2,
    CreateTask = 3,
    CreatePort = 4,
    VmAllocate = 5,
    CreateVmo = 6,
    VmFree = 7,
    MapVmo = 8,
    MapPhys = 9,
    TaskStart = 10,
    TaskGetVms = 11,
    CloseHandle = 12,
    PortCall = 13,
    PortSendWait = 14,
    PortReceive = 15,
    Maximum = 17,
}

impl TryFrom<usize> for SyscallList {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < Self::Maximum as usize {
            Ok(unsafe { core::mem::transmute::<usize, SyscallList>(value) })
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
