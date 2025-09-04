use bitmaps::{Bitmap, Bits, BitsImpl};

pub struct Ida<const N: usize>
where
    BitsImpl<N>: Bits,
{
    pool: Bitmap<N>,
}

impl<const N: usize> Ida<N>
where
    BitsImpl<N>: Bits,
{
    pub fn new() -> Self {
        Self {
            pool: Bitmap::new(),
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        let idx = self.pool.first_false_index()?;

        self.pool.set(idx, true);
        Some(idx)
    }

    pub fn free(&mut self, id: usize) {
        self.pool.set(id, false);
    }
}
