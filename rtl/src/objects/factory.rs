use bitflags::bitflags;

bitflags! {
    pub struct FactroryInvoke: usize {
        const CREATE_TASK = 0;
    }
}
