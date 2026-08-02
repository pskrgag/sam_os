//! Directory wrapper

use super::file::File;
use super::path::Path;
use crate::bindings_Vfs::{
    DirEntryFlagsFlag, Directory as BindingDirectory,
};
use alloc::{string::String, vec::Vec};
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

pub struct OpenOptions {
    pub create: bool,
}

pub struct DirEntry {
    pub name: String,
    pub is_directory: bool,
}

pub struct Directory {
    dir: BindingDirectory,
}

impl Directory {
    pub(crate) unsafe fn new(h: Handle) -> Result<Self, ErrorType> {
        unsafe {
            let port = Port::new(h);

            Ok(Self {
                dir: BindingDirectory::new(port),
            })
        }
    }

    pub fn try_clone(&self) -> Result<Self, ErrorType> {
        Ok(Self {
            dir: self.dir.try_clone()?,
        })
    }

    pub async fn list(&self) -> Result<Vec<DirEntry>, ErrorType> {
        Ok(self
            .dir
            .List()
            .await?
            .entries
            .into_iter()
            .map(|entry| DirEntry {
                name: String::from(entry.name.as_str()),
                is_directory: entry.flags == DirEntryFlagsFlag::Directory.into(),
            })
            .collect())
    }

    pub async fn open_file<'a, P: AsRef<Path<'a>>>(
        &self,
        path: P,
        options: OpenOptions,
    ) -> Result<File, ErrorType> {
        let path: &Path = path.as_ref();
        let path_str: &str = path.as_ref();
        let file = self
            .dir
            .OpenFile(
                path_str
                    .try_into()
                    .map_err(|_| ErrorType::InvalidArgument)?,
                options.create as _,
            )
            .await?;

        unsafe { File::new(file.handle) }
    }

    pub async fn open_dir<'a, P: AsRef<Path<'a>>>(
        &self,
        path: P,
        options: OpenOptions,
    ) -> Result<Self, ErrorType> {
        let path: &Path = path.as_ref();
        let path_str: &str = path.as_ref();
        let file = self
            .dir
            .OpenDir(
                path_str
                    .try_into()
                    .map_err(|_| ErrorType::InvalidArgument)?,
                options.create as _,
            )
            .await?;

        unsafe { Self::new(file.handle) }
    }
}
