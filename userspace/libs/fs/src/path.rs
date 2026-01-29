use alloc::string::{String, ToString};
use core::convert::AsRef;

#[repr(transparent)]
pub struct Path<'a> {
    inner: &'a str,
}

impl<'a> Path<'a> {
    pub fn new<S: AsRef<str>>(s: &'a S) -> Path<'a> {
        Self { inner: s.as_ref() }
    }

    pub fn into_owned(&self) -> String {
        self.inner.to_string()
    }

    pub fn parent(&'a self) -> Option<&'a Path<'a>> {
        self.inner
            .as_bytes()
            .iter()
            .rposition(|x| *x == b'/')
            .and_then(|idx| {
                self.inner.get(..self.inner.len() - idx - 1).map(|inner| {
                    // SAFETY
                    //
                    // seems sane, no?
                    unsafe { core::mem::transmute(&inner) }
                })
            })
    }
}

impl<'a> AsRef<str> for Path<'a> {
    fn as_ref(&self) -> &'a str {
        self.inner
    }
}

impl<'a> AsRef<Path<'a>> for &'a str {
    fn as_ref(&self) -> &Path<'a> {
        // SAFETY
        //
        // seems sane, no?
        unsafe { core::mem::transmute(self) }
    }
}
