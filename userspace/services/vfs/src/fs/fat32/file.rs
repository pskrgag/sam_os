use super::dir::Fat32DirRef;
use super::sb::Cluster;
use crate::bindings_Vfs::{File, FileRequest};
use crate::fs::inode::OpenFile;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use libc::vmm::{vm_object::VmObject, vms::vms};
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub struct FatFile {
    start: Option<Cluster>,
    parent: Fat32DirRef,
}

impl FatFile {
    pub fn new(start: Option<Cluster>, parent: Fat32DirRef) -> Result<OpenFile, ErrorType> {
        let port = Port::create()?;
        let raw_handle = port.handle().clone_handle()?;
        let s = Arc::new(Self { start, parent });

        let handler = File::for_each(port, move |req| {
            let file = s.clone();

            async move {
                match req {
                    FileRequest::Read { value, responder } => {
                        todo!()
                    }
                    FileRequest::Write { value, responder } => {
                        // SAFETY: hope that user does not fool us. If it did, then we will return
                        // an error.
                        let vmo = unsafe { VmObject::new(value.vmo) };
                        let ptr = vms().map_vm_object(&vmo, None, MappingType::RoData)?;

                        todo!()
                    }
                }

                Ok(())
            }
        });

        Ok(OpenFile {
            handler: Box::pin(handler),
            handle: raw_handle,
        })
    }
}
