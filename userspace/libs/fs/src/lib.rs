//! Wrappers for VFS
#![no_std]

use alloc::sync::Arc;
use libc::handle::Handle;
use rokio::port::Port;
use rtl::error::ErrorType;
use spin::Mutex;

extern crate alloc;

pub mod dir;
pub mod file;
pub mod path;

static CURRENT_DIR: Mutex<Option<Arc<dir::Directory>>> = Mutex::new(None);

/// # Safety
/// Caller must be sure handle is VFS handle
pub async unsafe fn init(handle: Handle) -> Result<(), ErrorType> {
    let vfs = bindings_Vfs::Vfs::new(unsafe { Port::new(handle) });
    let root = vfs.Root().await?;

    // we know that handle is root directory
    unsafe {
        chdir(dir::Directory::new(root.handle).unwrap());
    }

    Ok(())
}

pub fn cwd() -> Arc<dir::Directory> {
    let current = CURRENT_DIR.lock();

    current.as_ref().unwrap().clone()
}

pub fn chdir(dir: dir::Directory) {
    *CURRENT_DIR.lock() = Some(Arc::new(dir))
}

include!(concat!(env!("OUT_DIR"), "/nameserver.rs"));
include!(concat!(env!("OUT_DIR"), "/vfs.rs"));
