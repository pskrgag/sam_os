#![no_std]
#![no_main]

use bindings_BlkDev::BlkDev;
use bindings_NameServer::NameServer;
use fs::fat32::Fat32;
use fs::Filesystem;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

mod fs;
mod vfs;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let nameserver = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let root = nameserver.Get("blkdev".try_into().unwrap()).await?;
    let root = BlkDev::new(unsafe { Port::new(root.handle) });

    let mut vfs = vfs::Vfs::new(root, "fat32").await.unwrap();
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
