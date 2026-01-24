use alloc::string::{String, ToString};
use core::convert::AsRef;
use core::str::from_utf8;

pub struct Path {
    inner: [u8],
}

impl Path {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        unsafe { &*(s.as_ref() as *const _ as *const Path) }
    }

    pub fn into_owned(&self) -> String {
        from_utf8(&self.inner).unwrap().to_string()
    }
}

impl AsRef<Path> for &str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}
