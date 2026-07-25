use super::bindings_Vfs::Vfs;
use crate::bindings_Vfs::Directory;
use alloc::string::{String, ToString};
use core::ops::Deref;
use rokio::port::Port;
use rtl::error::ErrorType;

/// Current working directory
pub struct Cwd {
    dir: Directory,
    name: String,
}

impl Cwd {
    pub async fn root(vfs: &Vfs) -> Result<Self, ErrorType> {
        let dir = vfs.Root().await.unwrap();
        let dir = Directory::new(unsafe { Port::new(dir.handle) });

        Ok(Self {
            dir,
            name: "/".to_string(),
        })
    }

    pub fn new<S: AsRef<str>>(dir: Directory, name: S) -> Self {
        Self {
            dir,
            name: name.as_ref().to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Deref for Cwd {
    type Target = Directory;

    fn deref(&self) -> &Self::Target {
        &self.dir
    }
}
