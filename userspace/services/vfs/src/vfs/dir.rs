use crate::bindings_Vfs::{Directory, DirectoryRequest};
use crate::vfs::inode::{DirectoryOperations, Inode, InodeKind};
use crate::vfs::vfs;
use crate::vfs::Dentry;
use alloc::sync::Arc;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

pub struct OpenDirectory {
    dentry: Arc<Dentry>,
    ops: Arc<dyn DirectoryOperations>,
}

impl OpenDirectory {
    pub fn new(
        dentry: Arc<Dentry>,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        let port = Port::create()?;

        let ops = match dentry.inode().kind() {
            InodeKind::Directory(dir) => dir.clone(),
            _ => return Err(ErrorType::InvalidArgument),
        };

        let raw_handle = port.handle().clone_handle()?;
        let dir = Arc::new(Self { dentry, ops });

        Ok((
            Directory::for_each(port, move |req| {
                let dir = dir.clone();

                async move {
                    match req {
                        DirectoryRequest::List { responder, .. } => {
                            let res = dir.ops.list().await?;
                            let mut wire_res = heapless::Vec::new();

                            wire_res.extend_from_slice(&res).unwrap();
                            responder.reply(wire_res)?;
                        }
                        DirectoryRequest::OpenFile { value, responder } => {
                            let file = dir.dentry.lookup(&*value.name).await?;

                            if !file.inode().is_file() {
                                return Err(ErrorType::InvalidArgument);
                            }

                            let (handler, handle) =
                                super::file::OpenFile::new(file.inode().clone())?;

                            rokio::executor::spawn(handler);
                            responder.reply(&handle)?;
                        }
                        DirectoryRequest::CreateFile { value, responder } => {
                            let file = dir.dentry.lookup(&*value.name).await?;

                            if !file.inode().is_dir() {
                                return Err(ErrorType::InvalidArgument);
                            }

                            let (handler, handle) =
                                super::file::OpenFile::new(file.inode().clone())?;

                            rokio::executor::spawn(handler);
                            responder.reply(&handle)?;
                        }
                        DirectoryRequest::CreateDir { value, responder } => {

                        },
                    }

                    Ok(())
                }
            }),
            raw_handle,
        ))
    }
}
