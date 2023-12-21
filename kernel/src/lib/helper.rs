#[macro_export]
macro_rules! make_array {
    ($n:expr, $constructor:expr) => {
        #[allow(deprecated, invalid_value)]
        {
            let mut items: [_; $n] = core::mem::uninitialized();
            for (i, place) in items.iter_mut().enumerate() {
                core::ptr::write(place, $constructor(i));
            }
            items
        }
    };
}
