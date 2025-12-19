use crate::kernel::syscalls::SyscallArgs;
use aarch64_cpu::registers::{ESR_EL1, Readable};
use hal::address::VirtAddr;
use rtl::error::ErrorType;

unsafe extern "C" {
    // fn kernel_thread_entry_point();
    fn switch_to_user(ctx: *mut Context);
}

#[repr(usize)]
enum RawTrapReason {
    Irq = 0,
    Fiq = 1,
    DataAbort = 2,
    SError = 3,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TrapReason {
    Irq = 0,
    PageFault = 1,
    Syscall = 2,
    // TODO: more (and it should be arch independent)
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Context {
    pub x0: usize,
    pub x1: usize,
    pub x2: usize,
    pub x3: usize,
    pub x4: usize,
    pub x5: usize,
    pub x6: usize,
    pub x7: usize,
    pub x8: usize,
    pub x9: usize,
    pub x10: usize,
    pub x11: usize,
    pub x12: usize,
    pub x13: usize,
    pub x14: usize,
    pub x15: usize,
    pub x16: usize,
    pub x17: usize,
    pub x18: usize,
    pub x19: usize,
    pub x20: usize,
    pub x21: usize,
    pub x22: usize,
    pub x23: usize,
    pub x24: usize,
    pub x25: usize,
    pub x26: usize,
    pub x27: usize,
    pub x28: usize,
    pub x29: usize,
    pub elr: usize,
    pub spsr: usize,
    pub sp_el0: usize,
    pub x30: usize,
    pub reason: usize,
}

impl TryInto<SyscallArgs> for Context {
    type Error = ErrorType;

    fn try_into(self) -> Result<SyscallArgs, ErrorType> {
        SyscallArgs::new(
            self.x0,
            [
                self.x1, self.x2, self.x3, self.x4, self.x5, self.x6, self.x7,
            ],
        )
        .ok_or(ErrorType::InvalidArgument)
    }
}

impl Context {
    pub fn new(ep: VirtAddr, user_stack: VirtAddr, args: [usize; 3]) -> Self {
        let mut new: Self = unsafe { core::mem::zeroed() };

        new.elr = ep.into();
        new.sp_el0 = user_stack.into();
        new.x0 = args[0];
        new.x1 = args[1];
        new.x2 = args[2];

        new
    }

    pub unsafe fn switch(&mut self) {
        unsafe { switch_to_user(self) }
    }

    // TODO: actually it would be good to save ESR into the context
    pub fn trap_reason(&self) -> TrapReason {
        match self.reason {
            x if x == RawTrapReason::Fiq as usize || x == RawTrapReason::Irq as usize => {
                TrapReason::Irq
            }
            x if x == RawTrapReason::DataAbort as usize => {
                let ec = ESR_EL1.read(ESR_EL1::EC);

                // TODO: fucking tock_registers no idea how to use them
                if ec == 0b01_0101 {
                    TrapReason::Syscall
                } else {
                    TrapReason::PageFault
                }
            }
            _ => panic!("Corrupted context"),
        }
    }

    pub fn finish_syscall(&mut self, res: usize) {
        self.x0 = res;
    }
}
