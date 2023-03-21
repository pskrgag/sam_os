pub enum HandleType {
    Invalid,
    Untyped,
    PageTable,
    TCB,
}

pub enum HandleRight {
    Untyped,
    PageTable,
}

type UHandle = u64;
