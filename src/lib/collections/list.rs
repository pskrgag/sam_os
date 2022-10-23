use core::ptr::NonNull;

struct Node<T: ?Sized> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    data: T,
}

pub struct List<T: ?Sized> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
}

impl<T: ?Sized> List<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }


}
