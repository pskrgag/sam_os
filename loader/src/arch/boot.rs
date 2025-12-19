use aarch64_cpu::registers::{
    MAIR_EL1, ReadWriteable, SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1, Writeable,
};
use core::arch::asm;

pub fn boot(ep: usize, arg0: usize, tt: usize) {
    crate::println!("Booting kernel at ep {:x} and arg0 {:x}", ep, arg0);

    TCR_EL1.modify(
        TCR_EL1::T0SZ.val(16)
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::T1SZ.val(16)
            + TCR_EL1::SH0::Inner
            + TCR_EL1::SH1::Inner
            + TCR_EL1::TG1::KiB_4
            + TCR_EL1::IPS::Bits_40,
    );

    MAIR_EL1.modify(
        MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck,
    );

    TTBR1_EL1.set(tt as u64);
    TTBR0_EL1.set(tt as u64);

    SCTLR_EL1.modify(
        SCTLR_EL1::C::Cacheable
            + SCTLR_EL1::I::Cacheable
            + SCTLR_EL1::SA::Enable
            + SCTLR_EL1::M::Enable,
    );

    // From here MMU is ON. Don't use global vars and MMIO, since they are inaccessible
    unsafe {
        asm!(
            "mov x0, {x}",
            "mov x2, {ep}",
            "br x2",
            x = in(reg) arg0,
            ep = in(reg) ep,
        );
    }

    loop {}
}
