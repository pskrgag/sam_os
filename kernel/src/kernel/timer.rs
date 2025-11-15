// type CallbackFn<T> = fn(T);
//
// pub struct Timer<'a, T> {
//     delay: Option<usize>,
//     callback: CallbackFn<T>,
//     data: Option<&'a T>,
// }
//
// impl<'a, T> Timer<'a, T> {
//     pub fn new(cb: CallbackFn<T>) -> Self {
//         Self {
//             delay: None,
//             callback: cb,
//             data: None,
//         }
//     }
//
//     pub fn register(&mut self, delay: usize, data: Option<&'a T>) {
//         self.data = data;
//         self.delay = Some(delay);
//     }
// }
