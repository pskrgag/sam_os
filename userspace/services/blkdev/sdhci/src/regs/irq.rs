use bitmask::bitmask;

// Normal interrupt bits
bitmask! {
    pub mask SdhciIrqs: u16 where flags SdhciIrq {
        CommandCompleted = 1,
        TransactionCompleted = 2,
    }
}
