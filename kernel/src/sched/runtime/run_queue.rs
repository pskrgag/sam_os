use super::task::Task;
use super::waker::WakerPage;
use crate::adt::Vec;
use alloc::sync::Arc;
use core::num::NonZeroU32;
use core::task::Waker;
use rtl::error::ErrorType;
use slab::{Key as SlabKey, Slab};

pub struct RunQueue {
    slab: Slab<Arc<Task>>,
    wakers: Vec<WakerPage>,
}

pub struct TaskRef {
    pub task: Arc<Task>,
    pub waker: Waker,
}

impl RunQueue {
    pub fn new() -> Self {
        Self {
            slab: Slab::new(),
            wakers: Vec::new(),
        }
    }

    pub fn add(&mut self, t: Task) -> Result<(), ErrorType> {
        let new = Arc::try_new(t)
            .map_err(|_| ErrorType::NoMemory)
            .and_then(|x| self.slab.add(x).ok_or(ErrorType::NoMemory))?;
        let key = RQKey(unsafe { new.into_inner().into() });

        if key.0 as usize >= self.wakers.len() * WakerPage::num_entries() {
            self.wakers.try_push(WakerPage::new())?;
        }

        self.wakers[key.waker() as usize].initialize(key.waker_index());
        Ok(())
    }

    pub fn tasks(&mut self) -> impl Iterator<Item = TaskRef> {
        core::iter::from_coroutine(
            #[coroutine]
            || loop {
                for (i, page) in self.wakers.iter().enumerate() {
                    for notified in page.notified() {
                        let task_idx = unsafe {
                            SlabKey::from_u32(
                                NonZeroU32::new(
                                    (i * WakerPage::num_entries() + notified as usize) as u32,
                                )
                                .unwrap(),
                            )
                        };

                        let task = self.slab.get(&task_idx).unwrap();
                        yield TaskRef {
                            task: task.clone(),
                            waker: page.waker(notified),
                        };
                    }
                }
            },
        )
    }
}

pub struct RQKey(u32);

impl RQKey {
    pub fn task(&self) -> u32 {
        self.0
    }

    pub fn waker(&self) -> u32 {
        self.0 / WakerPage::num_entries() as u32
    }

    pub fn waker_index(&self) -> u8 {
        (self.0 % WakerPage::num_entries() as u32) as u8
    }
}
