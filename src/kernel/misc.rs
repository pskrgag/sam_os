extern "C" {
    static start: u64;
    static end: u64;
}

pub fn image_size() -> usize {
    unsafe { (&end as *const _) as usize - (&start as *const _) as usize}
}
