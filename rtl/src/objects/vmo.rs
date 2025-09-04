use bitflags::bitflags;

bitflags! {
    pub struct VmoFlags: usize {
        const BACKED = 0;
        const ZEROED = 1;
    }
}
