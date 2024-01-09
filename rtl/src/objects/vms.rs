use bitflags::bitflags;

bitflags! {
    pub struct VmsInvoke: usize {
        const ALLOCATE = 0;
        const CREATE_VMO = 1;
        const MAP_VMO = 2;
        const MAP_PHYS = 3;
    }
}
