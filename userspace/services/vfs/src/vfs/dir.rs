use crate::bindings_Vfs::{Directory, DirectoryRequest};
use crate::fs::inode::{DirectoryOperations, Inode};
use alloc::sync::Arc;
use rokio::port::Port;
use rtl::error::ErrorType;
use libc::handle::Handle;

pub struct OpenDirectory {
    inode: Arc<dyn DirectoryOperations>,
}

impl OpenDirectory {
    pub fn new(
        inode: Arc<Inode>,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        let port = Port::create()?;

        let inode = match &*inode {
            Inode::Directory(v) => v.clone(),
            _ => return Err(ErrorType::InvalidArgument)?,
        };
        let raw_handle = port.handle().clone_handle()?;
        let dir = Arc::new(Self { inode });

        Ok((
            Directory::for_each(port, move |req| {
                let dir = dir.clone();

                async move {
                    match req {
                        DirectoryRequest::Open { value, responder } => {
                            let new = dir.inode.open(&value.path)?;

                            todo!()
                        }
                    }

                    Ok(())
                }
            }),
            raw_handle,
        ))
    }
}
