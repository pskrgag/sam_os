use super::rdesc::Rdesc;
use super::tdesc::Tdesc;
use super::{Control, Rctl, Status, Tctl};
use crate::e1000::RxBuffer;
use crate::e1000::TxBuffer;
use core::ptr::NonNull;
use hal::address::{Address, VirtAddr, VirtualAddress};
use rtl::error::ErrorType;
use safe_mmio::UniqueMmioPointer;
use safe_mmio::{
    field,
    fields::{ReadOnly, ReadWrite},
};

pub enum E1000Error {
    MacInvalid,
    ResetTimeout,
    NoMemory,
}

impl From<E1000Error> for ErrorType {
    fn from(_: E1000Error) -> Self {
        ErrorType::InternalError
    }
}

#[repr(C, packed)]
struct E1000RegsRaw {
    pub ctrl: ReadWrite<Control>, // 0x0000
    pub _reserved_0004: [u8; 0x04],
    pub status: ReadOnly<Status>, // 0x0008
    pub _reserved_000c: [u8; 0x04],
    pub eecd: ReadWrite<u32>,     // 0x0010
    pub eerd: ReadWrite<u32>,     // 0x0014
    pub ctrl_ext: ReadWrite<u32>, // 0x0018
    pub fla: ReadWrite<u32>,      // 0x001c
    pub mdic: ReadWrite<u32>,     // 0x0020
    pub _reserved_0024: [u8; 0x04],
    pub fcal: ReadWrite<u32>, // 0x0028
    pub fcah: ReadWrite<u32>, // 0x002c
    pub fct: ReadWrite<u32>,  // 0x0030
    pub _reserved_0034: [u8; 0x04],
    pub vet: ReadWrite<u32>, // 0x0038
    pub _reserved_003c: [u8; 0x84],
    pub icr: ReadOnly<u32>,  // 0x00c0
    pub itr: ReadWrite<u32>, // 0x00c4
    pub ics: ReadWrite<u32>, // 0x00c8
    pub _reserved_00cc: [u8; 0x04],
    pub ims: ReadWrite<u32>, // 0x00d0
    pub _reserved_00d4: [u8; 0x04],
    pub imc: ReadWrite<u32>, // 0x00d8
    pub _reserved_00dc: [u8; 0x24],
    pub rctl: ReadWrite<Rctl>, // 0x0100
    pub _reserved_0104: [u8; 0x6c],
    pub fcttv: ReadWrite<u32>, // 0x0170
    pub _reserved_0174: [u8; 0x04],
    pub txcw: ReadWrite<u32>, // 0x0178
    pub rxcw: ReadOnly<u32>,  // 0x017c
    pub _reserved_0180: [u8; 0x280],
    pub tctl: ReadWrite<Tctl>,    // 0x0400
    pub tctl_ext: ReadWrite<u32>, // 0x0404
    pub tipg: ReadWrite<u32>,     // 0x0408
    pub _reserved_040c: [u8; 0xbf4],
    pub pba: ReadWrite<u32>, // 0x1000
    pub _reserved_1004: [u8; 0x115c],
    pub fcrtl: ReadWrite<u32>, // 0x2160
    pub _reserved_2164: [u8; 0x04],
    pub fcrth: ReadWrite<u32>, // 0x2168
    pub _reserved_216c: [u8; 0x694],
    pub rdbal: ReadWrite<u32>, // 0x2800
    pub rdbah: ReadWrite<u32>, // 0x2804
    pub rdlen: ReadWrite<u32>, // 0x2808
    pub _reserved_280c: [u8; 0x04],
    pub rdh: ReadWrite<u32>, // 0x2810
    pub _reserved_2814: [u8; 0x04],
    pub rdt: ReadWrite<u32>, // 0x2818
    pub _reserved_281c: [u8; 0x04],
    pub rdtr: ReadWrite<u32>, // 0x2820
    pub _reserved_2824: [u8; 0x08],
    pub radv: ReadWrite<u32>, // 0x282c
    pub _reserved_2830: [u8; 0xfd0],
    pub tdbal: ReadWrite<u32>, // 0x3800
    pub tdbah: ReadWrite<u32>, // 0x3804
    pub tdlen: ReadWrite<u32>, // 0x3808
    pub _reserved_380c: [u8; 0x04],
    pub tdh: ReadWrite<u32>, // 0x3810
    pub _reserved_3814: [u8; 0x04],
    pub tdt: ReadWrite<u32>, // 0x3818
    pub _reserved_381c: [u8; 0x04],
    pub tidv: ReadWrite<u32>, // 0x3820
    pub _reserved_3824: [u8; 0x08],
    pub tadv: ReadWrite<u32>, // 0x382c
    pub _reserved_3830: [u8; 0x1bd0],
    pub ral: ReadWrite<u32>, // 0x5400
    pub rah: ReadWrite<u32>, // 0x5404
}

pub struct E1000Regs(UniqueMmioPointer<'static, E1000RegsRaw>);

impl E1000Regs {
    pub fn new(
        va: VirtAddr,
        tx_buffer: &TxBuffer,
        rx_buffer: &RxBuffer,
    ) -> Result<Self, E1000Error> {
        let mut s =
            Self(unsafe { UniqueMmioPointer::new(NonNull::new_unchecked(va.to_raw_mut())) });

        s.initialize(tx_buffer, rx_buffer).map(|_| s)
    }

    pub fn set_rdt(&mut self, new: u32) {
        field!(self.0, rdt).modify_mut(|x| *x = new);
    }

    pub fn set_tdt(&mut self, new: u32) {
        field!(self.0, tdt).modify_mut(|x| *x = new);
    }

    fn reset(&mut self) -> Result<(), E1000Error> {
        let mut retries = 100;

        field!(self.0, ctrl).modify_mut(|x| *x = x.set(Control::RESET, true));

        while {
            if field!(self.0, ctrl).read().is_set(Control::RESET) {
                core::hint::spin_loop();
            } else {
                return Ok(());
            }

            retries -= 1;
            retries != 0
        } {}

        Err(E1000Error::ResetTimeout)
    }

    fn initialize(&mut self, tx: &TxBuffer, rx: &RxBuffer) -> Result<(), E1000Error> {
        // Things caller must ensure. This is invariant of DMA API anyway
        assert!(tx.ring_size() <= u32::MAX as usize);
        assert!(rx.ring_size() <= u32::MAX as usize);
        assert_eq!(
            rx.ring_size()
                .next_multiple_of(core::mem::size_of::<Rdesc>()),
            rx.ring_size()
        );
        assert_eq!(
            tx.ring_size()
                .next_multiple_of(core::mem::size_of::<Tdesc>()),
            tx.ring_size()
        );

        let rx_count = rx.ring_size() / core::mem::size_of::<Rdesc>() - 1;

        // Mask all IRQs
        field!(self.0, imc).write(u32::MAX);

        // Ack pending IRQs
        field!(self.0, icr).read();

        // Disable rx and tx
        field!(self.0, rctl).modify_mut(|x| *x = x.set(Rctl::ENABLE, false));
        field!(self.0, tctl).modify_mut(|x| *x = x.set(Tctl::ENABLE, false));

        self.reset()?;

        // Do it once again after reset
        field!(self.0, imc).write(u32::MAX);
        field!(self.0, icr).read();

        // Set RX buffers
        field!(self.0, rdlen).modify_mut(|x| *x = rx.ring_size() as u32);
        field!(self.0, rdbal).modify_mut(|x| *x = (rx.ring_pa().bits() & 0xFFFFFFFF) as u32);
        field!(self.0, rdbah)
            .modify_mut(|x| *x = ((rx.ring_pa().bits() >> 32) & 0xFFFFFFFF) as u32);
        field!(self.0, rdh).modify_mut(|x| *x = 0);
        field!(self.0, rdt).modify_mut(|x| *x = rx_count as u32);

        // Set TX buffers
        field!(self.0, tdlen).modify_mut(|x| *x = tx.ring_size() as u32);
        field!(self.0, tdbal).modify_mut(|x| *x = (tx.ring_pa().bits() & 0xFFFFFFFF) as u32);
        field!(self.0, tdbah)
            .modify_mut(|x| *x = ((tx.ring_pa().bits() >> 32) & 0xFFFFFFFF) as u32);
        field!(self.0, tdh).modify_mut(|x| *x = 0);
        field!(self.0, tdt).modify_mut(|x| *x = 0);

        // Don't filter broadcast packets
        field!(self.0, rctl).modify_mut(|x| *x = x.set(Rctl::BAM, true));
        // Set RX buffer params
        field!(self.0, rctl).modify_mut(|x| *x = x.set(Rctl::bsize(1 << rx.data_order()), true));
        // Enable receive
        field!(self.0, rctl).modify_mut(|x| *x = x.set(Rctl::ENABLE, true));

        // Pad short packets. Makes parsing easier
        field!(self.0, tctl).modify_mut(|x| *x = x.set(Tctl::PSP, true));
        // Use recommended value for collision_threshold.
        field!(self.0, tctl).modify_mut(|x| *x = x.set(Tctl::collision_threshold(0xf), true));
        // Use recommended value for collision_distance.
        field!(self.0, tctl).modify_mut(|x| *x = x.set(Tctl::collision_distance(0x40), true));
        // Enable transmit
        field!(self.0, tctl).modify_mut(|x| *x = x.set(Tctl::ENABLE, true));

        // Something default... TODO: figure out wtf is going on here
        field!(self.0, tipg).write(0x0060200a);

        // Configure link
        field!(self.0, ctrl).modify_mut(|x| *x = x.set(Control::SLU, true));
        assert!(field!(self.0, status).read().is_set(Status::LU));

        Ok(())
    }

    pub fn mac(&mut self) -> Result<u64, E1000Error> {
        let low = field!(self.0, ral).read();
        let mut high = field!(self.0, rah).read();

        if high & 1 << 31 == 0 {
            return Err(E1000Error::MacInvalid);
        }

        // Clear last (valid bit)
        high &= (1 << 16) - 1;
        Ok(((high as u64) << 32) | low as u64)
    }
}

const _: () = {
    assert!(core::mem::offset_of!(E1000RegsRaw, status) == 0x0008);
    assert!(core::mem::offset_of!(E1000RegsRaw, icr) == 0x00c0);
    assert!(core::mem::offset_of!(E1000RegsRaw, rctl) == 0x0100);
    assert!(core::mem::offset_of!(E1000RegsRaw, tctl) == 0x0400);
    assert!(core::mem::offset_of!(E1000RegsRaw, pba) == 0x1000);
    assert!(core::mem::offset_of!(E1000RegsRaw, rdbal) == 0x2800);
    assert!(core::mem::offset_of!(E1000RegsRaw, tdbal) == 0x3800);
    assert!(core::mem::offset_of!(E1000RegsRaw, ral) == 0x5400);
};
