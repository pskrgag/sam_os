use lock_free_buddy_allocator::cpuid;

pub struct Cpu;

impl cpuid::Cpu for Cpu {
    fn current_cpu() -> usize {
        0
    }
}
