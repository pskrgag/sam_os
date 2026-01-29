#![no_std]
#![no_main]

use bindings_BlkDev::BlkDev;
use bindings_NameServer::NameServer;
use bindings_Vfs::{Vfs, VfsRequest};
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;

mod fs;
mod vfs;

#[rokio::main]
async fn main(root: Option<Handle>) -> Result<(), ErrorType> {
    let ns = NameServer::new(unsafe { Port::new(root.unwrap()) });
    let root = ns.Get("blkdev".try_into().unwrap()).await?;
    let root = BlkDev::new(unsafe { Port::new(root.handle) });

    vfs::init(root, "fat32").await;

    let vfs = vfs::vfs();
    let port = Port::create()?;

    ns.Register("vfs".try_into().unwrap(), port.handle())
        .await
        .expect("Failed to register handle in nameserver");

    Vfs::for_each(port, move |req| {
        let vfs = vfs.clone();

        async move {
            match req {
                VfsRequest::Root { responder, .. } => {
                    let (disp, handle) = vfs.open_dir("/").await?;

                    println!("Open root");
                    rokio::executor::spawn(disp);
                    responder.reply(&handle)?;
                }
            }
            Ok(())
        }
    })
    .await
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
include!(concat!(env!("OUT_DIR"), "/vfs.rs"));
