use alloc::boxed::Box;

struct Node<T: Sized> {
    next: Option<Box<Node<T>>>,
    prev: Option<Box<Node<T>>>,
    data: T,
}

pub struct List<T: Sized> {
    head: Option<Box<Node<T>>>,
}

impl<T: Sized> List<T> {
    pub const fn new() -> Self {
        Self { head: None }
    }
}

impl<T: Sized> Node<T> {
    pub fn new(val: T) -> Self {
        Self {
            data: val,
            next: None,
            prev: None,
        }
    }
}
