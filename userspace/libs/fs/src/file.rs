//! Directory wrapper

use crate::bindings_Vfs::File as BindingFile;
use alloc::vec::Vec;
use hal::address::VirtualAddress;
use libc::factory::factory;
use libc::handle::Handle;
use libc::vmm::vms::vms;
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::vmm::MappingType;

pub struct File {
    file: BindingFile,
}

impl File {
    const READ_CHUNK_SIZE: usize = 1 << 12;

    pub(crate) unsafe fn new(h: Handle) -> Result<Self, ErrorType> {
        unsafe {
            let port = Port::new(h);

            Ok(Self {
                file: BindingFile::new(port),
            })
        }
    }

    pub async fn read(&self, data: &mut [u8]) -> Result<usize, ErrorType> {
        let vmo = factory().create_vm_object(data.len(), MappingType::Data)?;

        let read_len = self.file.Read(data.len(), vmo.handle()).await?.read;
        let buf = vms().map_vm_object(&vmo, None, MappingType::Data)?;
        let buf = unsafe { buf.as_slice(read_len) };

        data[..read_len].copy_from_slice(buf);
        Ok(read_len)
    }

    pub async fn read_to_end(&self) -> Result<Vec<u8>, ErrorType> {
        let mut result = Vec::new();
        let mut chunk = [0; Self::READ_CHUNK_SIZE];

        loop {
            let read = self.read(&mut chunk).await?;
            result.extend_from_slice(&chunk[..read]);

            if read < chunk.len() {
                return Ok(result);
            }
        }
    }

    pub async fn write(&self, data: &[u8]) -> Result<usize, ErrorType> {
        let vmo = factory().create_vm_object(data.len(), MappingType::Data)?;

        let mut buf = vms().map_vm_object(&vmo, None, MappingType::Data)?;
        let buf = unsafe { buf.as_slice_mut(data.len()) };

        buf.copy_from_slice(data);

        let write_len = self.file.Write(data.len(), vmo.handle()).await?.write;
        Ok(write_len)
    }
}
