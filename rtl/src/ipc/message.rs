use crate::handle::*;

pub const IPC_MAX_HANDLES: usize = 10;

#[derive(Debug)]
pub struct IpcMessage<'a> {
    handles: [Handle; IPC_MAX_HANDLES],
    num_handles: usize,
    user_data: Option<&'a [u8]>,
}

impl<'a> IpcMessage<'a> {
    const DEFAULT: Self = Self {
        handles: [HANDLE_INVALID; IPC_MAX_HANDLES],
        num_handles: 0,
        user_data: None,
    };

    pub const fn new() -> Self { Self::DEFAULT }

    pub fn handles(&self) -> &[HandleBase] {
        &self.handles[..self.num_handles]
    }

    pub fn data(&self) -> Option<&[u8]> {
        self.user_data
    }

    pub fn add_data<T: AsRef<[u8]> + ?Sized>(&mut self, data: &'a T) {
        assert!(self.user_data.is_none());
        self.user_data = Some(data.as_ref());
    }

    pub fn add_data_raw(&mut self, data: &'a [u8]) {
        assert!(self.user_data.is_none());
        self.user_data = Some(data);
    }

    pub fn add_handle(&mut self, h: Handle) {
        // TODO: that API should not cause a crash
        assert!(self.num_handles != IPC_MAX_HANDLES);

        self.handles[self.num_handles] = h;
        self.num_handles += 1;
    }

    pub fn add_handles(&mut self, h: &[Handle]) {
        // TODO: that API should not cause a crash
        assert!(self.num_handles != IPC_MAX_HANDLES);

        for i in h {
            self.add_handle(*i);
        }
    }
}

impl Default for IpcMessage<'_> {
    fn default() -> Self {
        Self::DEFAULT
    }
}
