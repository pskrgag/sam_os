use crate::bindings_Vfs::{File, FileRequest};
use crate::vfs::inode::{FileOperations, Inode, InodeKind};
use alloc::sync::Arc;
use hal::address::VirtualAddress;
use libc::handle::Handle;
use libc::vmm::vm_object::VmObject;
use libc::vmm::vms::vms;
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;
use rtl::vmm::MappingType;

pub struct OpenFile {
    inode: Arc<Inode>,
    local_offset: usize,
    ops: Arc<dyn FileOperations>,
}

impl OpenFile {
    pub fn new(
        inode: Arc<Inode>,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        let port = Port::create()?;

        let ops = match inode.kind() {
            InodeKind::File(dir) => dir.clone(),
            _ => return Err(ErrorType::InvalidArgument),
        };

        let raw_handle = port.handle().clone_handle()?;
        let file = Arc::new(Spinlock::new(Self {
            inode,
            ops,
            local_offset: 0,
        }));

        Ok((
            File::for_each(port, move |req| {
                let file = file.clone();

                async move {
                    match req {
                        FileRequest::Read { value, responder } => {
                            let mut file = file.lock();
                            let vmo = unsafe { VmObject::new(value.vmo) };
                            let mut buf = vms().map_vm_object(&vmo, None, MappingType::Data)?;
                            // TODO: this is really unsafe and we should check the size of the VMO
                            // and do not believe the user.
                            let buf = unsafe { buf.as_slice_mut(value.size) };

                            let res = file.ops.read(buf, file.local_offset + value.offset).await?;
                            println!("Read = {}", res);
                            file.local_offset += value.size;
                            responder.reply(res)?;
                        }
                        FileRequest::Write { value, responder } => {
                            let mut file = file.lock();

                            let vmo = unsafe { VmObject::new(value.vmo) };
                            let buf = vms().map_vm_object(&vmo, None, MappingType::RoData)?;
                            // TODO: this is really unsafe and we should check the size of the VMO
                            // and do not believe the user.
                            let buf = unsafe { buf.as_slice(value.size) };

                            let _res = file
                                .ops
                                .write(buf, file.local_offset + value.offset)
                                .await?;
                            file.local_offset += value.size;
                            responder.reply()?;
                        }
                    }

                    Ok(())
                }
            }),
            raw_handle,
        ))
    }
}
