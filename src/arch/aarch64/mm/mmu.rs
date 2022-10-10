// FIXME one day...
#[path = "../qemu/config.rs"]
mod config;

use crate::arch::mm::initial_map;
use crate::mm::types::MemRange;

use tock_registers::interfaces::Writeable;
use cortex_a::{asm, registers::*};

pub fn init() {
    for i in &config::MemoryLayout {
        unsafe { initial_map::initial_tt.populate_indential(&MemRange::new(i.start.into(), i.size)); }
    }

    unsafe { TTBR0_EL1.set_baddr((&initial_map::initial_tt as *const _) as u64); };
    println!("Set TTBR0");
}
