#[allow(dead_code)]
#[allow(unused_variables)]

use crate::arch;

#[derive(Debug)]
pub enum PaError {
    ENOMEM,
}

pub type Paddr = u64;
pub type PaResult = Result<Paddr, PaError>;

struct PageAlloc {
   freelist: *mut u8,
   pool: [u8; 10], 
}

pub static mut PageAlloc PageAllocator;

fn mm_set_up_page_allocator(ram_mem: &arch::MemoryRegion)
{

}

pub fn mm_set_up_memory_layout(memory_layout: &[arch::MemoryRegion]) {
    for i in memory_layout {
        match i.tp {
           arch::MemoryType::MEM => mm_set_up_page_allocator(i),
           _ => {},
        }
    }
}
