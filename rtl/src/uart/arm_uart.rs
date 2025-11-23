use crate::uart::UartTrait;
use hal::address::VirtAddr;
use core::mem::size_of;
use core::ptr;

#[repr(u8)]
#[allow(dead_code)]
enum Pl011 {
    Dr = 0x000,
    Fr = 0x18,
    Rc = 0x30,
}

#[repr(u8)]
#[allow(dead_code)]
enum UartOpri {
    Lsr = 0x14,
}

const UARTFR_RXFE: u32 = 1 << 4;

const UART_CR_RXE: u32 = 1 << 9;
const UART_CR_TXE: u32 = 1 << 8;
const UART_CR_UARTEN: u32 = 1 << 0;

pub struct Uart {
    base: VirtAddr,
}

impl Uart {
    pub const fn invalid() -> Self {
        Self {
            base: VirtAddr::new(0),
        }
    }

    pub fn enable(&mut self) {
        self.write_reg(Pl011::Rc as u8, UART_CR_UARTEN | UART_CR_TXE | UART_CR_RXE);
    }

    pub fn new(base: VirtAddr) -> Self {
        let mut s = Self { base };
        s.enable();
        s
    }

    fn write_reg(&mut self, reg: u8, data: u32) {
        let ptr = self.base.to_raw_mut::<u32>();
        unsafe { ptr::write_volatile(ptr.add(reg as usize / size_of::<u32>()), data) };
    }

    fn read_reg(&self, reg: u8) -> u32 {
        let ptr = self.base.to_raw_mut::<u32>();
        unsafe { ptr::read_volatile(ptr.add(reg as usize / size_of::<u32>())) }
    }
}

impl UartTrait for Uart {
    fn base(&self) -> VirtAddr {
        self.base
    }

    fn read_bytes(&mut self, bytes: &mut [u8]) {
        for i in bytes {
            while self.read_reg(Pl011::Fr as u8) & UARTFR_RXFE > 0 {}

            *i = self.read_reg(Pl011::Dr as u8) as u8;
        }
    }

    fn write_bytes(&mut self, b: &[u8]) {
        for i in b {
            self.write_reg(Pl011::Dr as u8, *i as u32);

            if *i == b'\n' {
                self.write_reg(Pl011::Dr as u8, b'\r' as u32);
            }
        }
    }
}
