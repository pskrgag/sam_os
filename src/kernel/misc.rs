extern "C" {
    static start: u64;
    static end: u64;
}

pub fn image_size() -> u64 {
    unsafe { (&end as *const _) as u64  - (&start as *const _) as u64}
}
