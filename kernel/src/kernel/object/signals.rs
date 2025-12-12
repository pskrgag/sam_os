use bitmask::bitmask;

bitmask! {
    pub mask Signals: u8 where flags Signal {
        None = 0,
        Ready = 1,
    }
}

impl Default for Signals {
    fn default() -> Self {
        Signals::from(Signal::None)
    }
}
