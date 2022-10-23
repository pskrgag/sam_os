// FIXME one day...
#[path = "../arch/aarch64/qemu/config.rs"]
mod config;

use bitmaps::Bitmap;

struct MemoryRegion {

}

struct FFAlloc {
    size: usize,
}

impl FFAlloc {
    
}

fn init() -> Result<(), ()> {
    Ok(())
}
