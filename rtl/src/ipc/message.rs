use crate::handle::*;

pub const IPC_MAX_HANDLES: usize = 5;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IpcMessage<'a> {
    handles: [Handle; IPC_MAX_HANDLES],
    num_handles: usize,
    in_data: Option<&'a [u8]>,
    out_data: Option<&'a [u8]>,
    reply_port: Handle,
}

impl<'a> IpcMessage<'a> {
    const DEFAULT: Self = Self {
        handles: [HANDLE_INVALID; IPC_MAX_HANDLES],
        num_handles: 0,
        in_data: None,
        out_data: None,
        reply_port: HANDLE_INVALID,
    };

    pub const fn new() -> Self {
        Self::DEFAULT
    }

    pub fn handles(&self) -> &[HandleBase] {
        &self.handles[..self.num_handles]
    }

    pub fn in_data(&self) -> Option<&[u8]> {
        self.in_data
    }

    pub fn out_data(&self) -> Option<&[u8]> {
        self.out_data
    }

    pub fn reply_port(&self) -> HandleBase {
        self.reply_port
    }

    pub fn set_reply_port(&mut self, h: HandleBase) {
        self.reply_port = h;
    }

    pub fn add_data<T: Copy>(&mut self, data: &'a T) {
        assert!(self.in_data.is_none());
        self.in_data = Some(unsafe {
            core::slice::from_raw_parts(data as *const _ as *const u8, core::mem::size_of::<T>())
        });
    }

    pub fn add_data_raw(&mut self, data: &'a [u8]) {
        assert!(self.in_data.is_none());
        self.in_data = Some(data);
    }

    pub fn set_out_data<T: Copy>(&mut self, data: &'a T) {
        assert!(self.out_data.is_none());
        self.out_data = Some(unsafe {
            core::slice::from_raw_parts(data as *const _ as *const u8, core::mem::size_of::<T>())
        });
    }

    pub fn set_out_data_raw(&mut self, data: &'a [u8]) {
        assert!(self.out_data.is_none());
        self.out_data = Some(data);
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
