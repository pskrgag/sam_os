use crate::uart::UartTrait;
use crate::vmm::types::VirtAddr;
use core::mem::size_of;
use core::ptr;

#[repr(u8)]
#[allow(dead_code)]
enum Pl011 {
    UARTDR = 0x000,
    UARTFR = 0x18,
    UARTIBRD = 0x24,
    UARTFBRD = 0x28,
    UARTLCR = 0x2c,
    UARTCR = 0x30,
    UARTIMSC = 0x38,
    UARTICR = 0x44,
    UARTDMACR = 0x48,
}

#[repr(u8)]
#[allow(dead_code)]
enum UartOpri {
    UARTLSR = 0x14,
}

const UARTFR_RXFE: u32 = 1 << 4;

const UART_CR_RXE: u32 = 1 << 9;
const UART_CR_TXE: u32 = 1 << 8;
const UART_CR_UARTEN: u32 = 1 << 0;

pub struct Uart {
    base: VirtAddr,
}

impl Uart {
    pub const fn default(base: VirtAddr) -> Self {
        Self { base }
    }

    pub fn enable(&mut self) {
        self.write_reg(
            Pl011::UARTCR as u8,
            UART_CR_UARTEN | UART_CR_TXE | UART_CR_RXE,
        );
    }

    pub fn init(base: VirtAddr) -> Self {
        let mut s = Self::default(base);
        s.enable();
        s
    }

    fn write_reg(&mut self, reg: u8, data: u32) {
        let ptr = self.base.to_raw_mut::<u32>();
        unsafe {
            ptr::write_volatile(ptr.offset((reg as usize / size_of::<u32>()) as isize), data)
        };
    }

    fn read_reg(&self, reg: u8) -> u32 {
        let ptr = self.base.to_raw_mut::<u32>();
        unsafe { ptr::read_volatile(ptr.offset((reg as usize / size_of::<u32>()) as isize)) }
    }
}

impl UartTrait for Uart {
    fn base(&self) -> VirtAddr {
        self.base
    }

    fn read_bytes(&mut self, bytes: &mut [u8]) {
        for i in bytes {
            if env!("BOARD_TYPE") == "qemu" {
                while self.read_reg(Pl011::UARTFR as u8) & UARTFR_RXFE > 0 {}
            } else if env!("BOARD_TYPE") == "orpipc2" {
                while self.read_reg(UartOpri::UARTLSR as u8) & 0x1 == 0 {}
            }

            *i = self.read_reg(Pl011::UARTDR as u8) as u8;
        }
    }

    fn write_bytes(&mut self, b: &[u8]) {
        for i in b {
            if env!("BOARD_TYPE") == "orpipc2" {
                while self.read_reg(UartOpri::UARTLSR as u8) & 0x40 == 0 {}
            }

            self.write_reg(Pl011::UARTDR as u8, *i as u32);

            if *i == b'\n' {
                self.write_reg(Pl011::UARTDR as u8, b'\r' as u32);
            }
        }
    }
}
