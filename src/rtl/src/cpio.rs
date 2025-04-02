use core::fmt::{self, Debug};
use core::mem::size_of;
use core::slice;

// Using new ASCII format
const CPIO_MAGIC: [u8; 6] = *b"070701";
const CPIO_END: &str = "TRAILER!!!\0";

#[repr(C)]
struct CpioHeader {
    magic: [u8; 6],
    c_ino: [u8; 8],
    c_mode: [u8; 8],
    c_uid: [u8; 8],
    c_gid: [u8; 8],
    c_nlink: [u8; 8],
    c_mtime: [u8; 8],
    c_filesize: [u8; 8],
    c_devmajor: [u8; 8],
    c_devminor: [u8; 8],
    c_rdevmajor: [u8; 8],
    c_rdevminor: [u8; 8],
    c_namesize: [u8; 8],
    c_check: [u8; 8],
}

pub struct File<'a> {
    name: &'a str,
    data: &'a [u8],
}

pub struct Iter<'a> {
    archive: &'a Cpio<'a>,
    offset: usize,
}

pub struct Cpio<'a> {
    data: &'a [u8],
}

impl<'a> Cpio<'a> {
    pub fn new(data: &'a [u8]) -> Option<Self> {
        if data.len() < size_of::<CpioHeader>() {
            return None;
        }

        Some(Self { data })
    }

    pub fn iter(&'a self) -> Iter<'a> {
        Iter {
            archive: self,
            offset: 0,
        }
    }
}

impl CpioHeader {
    pub fn filesize(&self) -> usize {
        Self::ascii_to_number(&self.c_filesize)
    }

    /// Caller must make sure we don't go OOB
    pub unsafe fn filename(&self) -> &str {
        let ptr = (self as *const CpioHeader).offset(1) as *const u8;

        let slice = slice::from_raw_parts(ptr, Self::ascii_to_number(&self.c_namesize));

        core::str::from_utf8(slice).unwrap()
    }

    fn ascii_to_number(arr: &[u8]) -> usize {
        let size_slice = unsafe { slice::from_raw_parts(arr.as_ptr(), arr.len()) };
        let size_slice = core::str::from_utf8(size_slice).unwrap();

        usize::from_str_radix(size_slice, 16).unwrap()
    }
}

impl<'a> File<'a> {
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn name(&self) -> &str {
        self.name
    }
}

impl<'a> Iterator for Iter<'a> {
    // We can refer to this type using Self::Item
    type Item = File<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + size_of::<CpioHeader>() > self.archive.data.len() {
            return None;
        }

        let header = unsafe {
            (self.archive.data.as_ptr().add(self.offset) as *const CpioHeader)
                .as_ref()
                .unwrap()
        };

        if header.magic != CPIO_MAGIC {
            return None;
        }

        let name = unsafe { header.filename() };

        if name == CPIO_END {
            return None;
        }

        let file_size = header.filesize();
        let offset = self.offset + size_of::<CpioHeader>() + name.len();
        let offset = offset.next_multiple_of(4);

        let ret = Some(File {
            name,
            data: &self.archive.data[offset..offset + file_size],
        });

        self.offset = offset + file_size;

        ret
    }
}

impl<'a> Debug for File<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").field("name", &self.name).finish()
    }
}
