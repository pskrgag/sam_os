use crate::handle::*;

pub const IPC_MAX_HANDLES: usize = 5;

type MessageId = usize;

// Note: I want to keep Copy marker here, so I had to lie about
// mutablity of arena slices.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IpcMessage<'a> {
    handles: [Handle; IPC_MAX_HANDLES],
    num_handles: usize,
    data: Option<&'a [u8]>,
    reply_port: Handle,
    mid: MessageId,
}

impl<'a> IpcMessage<'a> {
    const DEFAULT: Self = Self {
        handles: [HANDLE_INVALID; IPC_MAX_HANDLES],
        num_handles: 0,
        data: None,
        reply_port: HANDLE_INVALID,
        mid: MessageId::MAX,
    };

    pub const fn empty() -> Self {
        Self::DEFAULT
    }

    pub fn new(data: &'a [u8], mid: MessageId, handles: &[Handle]) -> Self {
        let mut s = Self::default();

        assert!(handles.len() < 10);

        s.add_handles(handles);
        s.data = Some(data);
        s.reply_port = HANDLE_INVALID;
        s.mid = mid;

        s
    }

    pub fn new_receive(data: &'a mut [u8]) -> Self {
        let mut s = Self::default();

        s.data = Some(data);
        s
    }

    pub fn data(&self) -> Option<&'a [u8]> {
        self.data
    }

    pub fn data_mut(&mut self) -> Option<&'a mut [u8]> {
        unsafe {
            let data = self.data?;

            let ptr = data.as_ptr() as usize as *mut u8;

            Some(core::slice::from_raw_parts_mut(ptr, data.len()))
        }
    }

    pub fn set_reply_port(&mut self, reply_port: Handle) {
        self.reply_port = reply_port;
    }

    pub fn mid(&self) -> MessageId {
        self.mid
    }

    pub fn handles(&self) -> &[HandleBase] {
        &self.handles[..self.num_handles]
    }

    pub fn add_handles(&mut self, h: &[Handle]) {
        // TODO: that API should not cause a crash
        assert!(self.num_handles != IPC_MAX_HANDLES);

        for i in h {
            self.handles[self.num_handles] = *i;
            self.num_handles += 1;
        }
    }

    pub fn reply_port(&self) -> HandleBase {
        self.reply_port
    }
}

impl Default for IpcMessage<'_> {
    fn default() -> Self {
        Self::DEFAULT
    }
}
