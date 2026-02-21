use crate::bindings_Vfs::{Directory, DirectoryRequest};
use crate::vfs::inode::{DirectoryOperations, Inode, InodeKind};
use crate::vfs::vfs;
use alloc::sync::Arc;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

pub struct OpenDirectory {
    inode: Arc<Inode>,
    ops: Arc<dyn DirectoryOperations>,
}

impl OpenDirectory {
    pub fn new(
        inode: Arc<Inode>,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        let port = Port::create()?;

        let ops = match inode.kind() {
            InodeKind::Directory(dir) => dir.clone(),
            _ => return Err(ErrorType::InvalidArgument),
        };

        let raw_handle = port.handle().clone_handle()?;
        let dir = Arc::new(Self { inode, ops });

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
                            let file = if let Some(file) = vfs().dcache_lookup(&*value.name) {
                                file
                            } else {
                                let file = dir
                                    .ops
                                    .lookup((&value.name.as_str()).as_ref(), &dir.inode)
                                    .await?;
                                vfs().dcache_store(&*value.name, file)
                            };

                            if !file.is_file() {
                                return Err(ErrorType::InvalidArgument);
                            }

                            let (handler, handle) = super::file::OpenFile::new(file)?;

                            rokio::executor::spawn(handler);
                            responder.reply(&handle)?;
                        }
                        DirectoryRequest::CreateFile { value, responder } => {
                            let file = if let Some(file) = vfs().dcache_lookup(&*value.name) {
                                file
                            } else {
                                let file = dir.ops.create_file(&*value.name, &dir.inode).await?;
                                vfs().dcache_store(&*value.name, file)
                            };

                            let (handler, handle) = super::file::OpenFile::new(file)?;

                            rokio::executor::spawn(handler);
                            responder.reply(&handle)?;
                        }
                    }

                    Ok(())
                }
            }),
            raw_handle,
        ))
    }
}
