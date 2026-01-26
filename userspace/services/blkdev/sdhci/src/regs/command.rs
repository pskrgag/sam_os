use crate::regs::{Response, NoResponse};
use bitmask::bitmask;
use core::marker::PhantomData;

// https://users.ece.utexas.edu/~valvano/EE345M/SD_Physical_Layer_Spec.pdf
#[repr(u8)]
pub enum NormalOpcode {
    GoIdleState = 0,
    CID = 2,
    CSD = 9,
    RCA = 3,
    SelectCard = 7,
    VoltageCheck = 8,
    ReadOneBlock = 17,
    WriteOneBlock = 24,
    AppCmd = 55,
}

#[repr(u8)]
pub enum ApplicationOpcode {
    OpCond = 41,
}

bitmask! {
    pub mask CommandFlags: u8 where flags CommandFlag {
        NoResponse = 0,
        HasReponse = 1,
        DataPresent = 1 << 5,
    }
}

pub struct Command<R: Response = NoResponse> {
    opcode: u8,
    arg: u32,
    flags: CommandFlags,
    app: bool,
    _pd: PhantomData<R>,
}

impl<R: Response> Command<R> {
    pub fn new_normal<F: Into<CommandFlags>>(opcode: NormalOpcode, arg: u32, flags: F) -> Self {
        Self {
            opcode: opcode as u8,
            arg,
            flags: flags.into(),
            app: false,
            _pd: PhantomData,
        }
    }

    pub fn new_application<F: Into<CommandFlags>>(
        opcode: ApplicationOpcode,
        arg: u32,
        flags: F,
    ) -> Self {
        Self {
            opcode: opcode as u8,
            arg,
            flags: flags.into(),
            app: true,
            _pd: PhantomData,
        }
    }

    pub fn is_application(&self) -> bool {
        self.app
    }

    pub fn arg(&self) -> u32 {
        self.arg
    }

    pub fn need_response(&self) -> bool {
        *(self.flags & CommandFlag::HasReponse) != 0
    }
}

impl<R: Response> Into<u16> for Command<R> {
    fn into(self) -> u16 {
        // 13-08 -- command index (opcode)
        // 07-06 -- command type
        // 05-05 -- data present
        // 04-04 -- command index check enable
        // 03-03 -- command crc check enable
        // 02-02 -- subcommand check
        // 01-00 -- response type
        let response: u16 = (*self.flags).into();

        (self.opcode as u8 as u16) << 8 | response
    }
}
