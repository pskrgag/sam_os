use core::mem::{transmute, size_of};

const CPIO_MAGIC: [u8; 6] = *b"070707";

#[repr(C)]
struct CpioHeader {
    magic: [u8; 6],
    dev: [u8; 6],
    inode: [u8; 6],
    mode: [u8; 6],
    uid: [u8; 6],
    gid: [u8; 6],
    nlink: [u8; 6],
    rdev: [u8; 6],
    mtime: [u8; 11],
    namesize: [u8; 6],
    filesize: [u8; 11],
}

pub struct Cpio<'a> {
    header: &'a CpioHeader,
    data: &'a [u8]
}

impl<'a> Cpio<'a> {
    pub fn new(data: &'a [u8]) -> Option<Self> {
        if data.len() < size_of::<CpioHeader>() {
            return None;
        }

        let header = unsafe { (data.as_ptr() as *const CpioHeader).as_ref().unwrap() };

        if header.magic != CPIO_MAGIC {
            return None;
        }

        Some(Self {
            header: header,
            data: data
        })
    }
}
