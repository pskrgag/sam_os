use core::mem::variant_count;

#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum SyscallList {
    Write = 0,
    Yield = 1,
    CreateTask = 2,
    CreatePort = 3,
    VmAllocate = 4,
    CreateVmo = 5,
    VmFree = 6,
    MapVmo = 7,
    MapPhys = 8,
    TaskStart = 9,
    TaskGetVms = 10,
    CloseHandle = 11,
    PortCall = 12,
    PortSendWait = 13,
    PortReceive = 14,
    CloneHandle = 15,
    MapFdt = 16,
    PortSend = 17,
    WaitObject = 18,
    WaitObjectMany = 19,
}

impl TryFrom<usize> for SyscallList {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < variant_count::<Self>() {
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
