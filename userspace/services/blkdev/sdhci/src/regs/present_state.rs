use bitmask::bitmask;

// Normal interrupt bits
bitmask! {
    pub mask PresentState: u32 where flags PresentStateFlag  {
        SpaceAvailable = 0x00000400,
    }
}

