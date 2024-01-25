use crate::handle::*;

pub const IPC_MAX_HANDLES: usize = 5;

type MessageId = usize;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IpcMessage<'a> {
    handles: [Handle; IPC_MAX_HANDLES],
    num_handles: usize,
    in_data: Option<&'a [u8]>,
    out_data: Option<&'a [u8]>,
    reply_port: Handle,
    mid: MessageId,
}

impl<'a> IpcMessage<'a> {
    const DEFAULT: Self = Self {
        handles: [HANDLE_INVALID; IPC_MAX_HANDLES],
        num_handles: 0,
        in_data: None,
        out_data: None,
        reply_port: HANDLE_INVALID,
        mid: MessageId::MAX,
    };

    pub const fn new() -> Self {
        Self::DEFAULT
    }

    pub fn set_mid(&mut self, mid: MessageId) {
        self.mid = mid;
    }

    pub fn mid(&self) -> MessageId {
        self.mid
    }

    pub fn handles(&self) -> &[HandleBase] {
        &self.handles[..self.num_handles]
    }

    pub fn in_arena(&self) -> Option<&[u8]> {
        self.in_data
    }

    pub fn out_arena(&self) -> Option<&[u8]> {
        self.out_data
    }

    pub fn reply_port(&self) -> HandleBase {
        self.reply_port
    }

    pub fn set_reply_port(&mut self, h: HandleBase) {
        self.reply_port = h;
    }

    pub fn set_in_arena(&mut self, data: &'a [u8]) {
        // assert!(self.in_data.is_none());
        self.in_data = Some(data);
    }

    pub fn set_out_arena(&mut self, data: &'a [u8]) {
        // assert!(self.out_data.is_none());
        self.out_data = Some(data);
    }

    pub fn add_handle(&mut self, h: Handle) -> usize {
        // TODO: that API should not cause a crash
        assert!(self.num_handles != IPC_MAX_HANDLES);

        self.handles[self.num_handles] = h;
        self.num_handles += 1;

        self.num_handles - 1
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
