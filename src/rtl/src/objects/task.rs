use bitflags::bitflags;

bitflags! {
    pub struct TaskInvoke: usize {
        const START = 0;
        const GET_VMS = 1;
    }
}
