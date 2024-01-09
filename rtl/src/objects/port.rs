use bitflags::bitflags;

bitflags! {
    pub struct PortInvoke: usize {
        const RECEIVE = 0;
        const CALL = 1;
        const SEND = 2;
    }
}
