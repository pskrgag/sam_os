use alloc::string::{String, ToString};

#[derive(Clone, Copy)]
pub struct Path<'a> {
    inner: &'a str,
}

pub struct Components<'a> {
    inner: Option<&'a str>,
}

impl<'a> Iterator for Components<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.inner?;
        let inner = inner.trim_start_matches('/');

        if inner.is_empty() {
            self.inner = None;
            return None;
        }

        let pos = inner.find('/').unwrap_or(inner.len());
        let res = &inner[..pos];

        self.inner = inner.get(pos + 1..);
        Some(res)
    }
}

impl<'a> Path<'a> {
    pub fn new<S: AsRef<str>>(s: &'a S) -> Path<'a> {
        Self { inner: s.as_ref() }
    }

    pub fn components(&self) -> Components<'a> {
        Components {
            inner: Some(self.inner),
        }
    }

    pub fn into_owned(&self) -> String {
        self.inner.to_string()
    }

    pub fn skip_dir(&self) -> Path<'a> {
        let inner = self.inner.trim_start_matches('/');
        let pos = inner.find('/').unwrap_or(inner.len());
        let inner = inner.get(pos + 1..).unwrap_or("");

        Self { inner }
    }

    pub fn parent(&self) -> Option<Path<'a>> {
        let inner = self.inner.trim_end_matches('/');

        if inner.is_empty() || inner == "/" {
            return None;
        }

        let idx = inner.rfind('/')?;
        let parent = if idx == 0 { "/" } else { &inner[..idx] };

        Some(Self { inner: parent })
    }
}

impl<'a> AsRef<str> for Path<'a> {
    fn as_ref(&self) -> &'a str {
        self.inner
    }
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(value: &'a str) -> Self {
        Self { inner: value }
    }
}
