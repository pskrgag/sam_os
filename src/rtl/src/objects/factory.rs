use bitflags::bitflags;

bitflags! {
    pub struct FactroryInvoke: usize {
        const CREATE_TASK = 0;
        const CREATE_PORT = 1;
    }
}
