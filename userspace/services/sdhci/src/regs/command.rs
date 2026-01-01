use bitmask::bitmask;

// https://users.ece.utexas.edu/~valvano/EE345M/SD_Physical_Layer_Spec.pdf
#[repr(u8)]
pub enum NormalOpcode {
    GoIdleState = 0,
    CID = 2,
    RCA = 3,
    VoltageCheck = 8,
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
    }
}

pub struct Command {
    opcode: u8,
    arg: u32,
    flags: CommandFlags,
    app: bool,
}

impl Command {
    pub fn new_normal(opcode: NormalOpcode, arg: u32, flags: CommandFlags) -> Self {
        Self {
            opcode: opcode as u8,
            arg,
            flags,
            app: false,
        }
    }

    pub fn new_application(opcode: ApplicationOpcode, arg: u32, flags: CommandFlags) -> Self {
        Self {
            opcode: opcode as u8,
            arg,
            flags,
            app: true,
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

impl Into<u16> for Command {
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
