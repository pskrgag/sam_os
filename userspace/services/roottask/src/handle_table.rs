use alloc::vec::Vec;
use alloc::{collections::btree_map::BTreeMap, string::String};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use libc::handle::Handle;
use rtl::locking::spinlock::Spinlock;

pub struct HandleTable {
    table: Spinlock<BTreeMap<String, Handle>>,
    waiters: Spinlock<Vec<Waker>>,
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            table: Spinlock::new(BTreeMap::new()),
            waiters: Spinlock::new(Vec::new()),
        }
    }

    pub fn insert(&self, name: String, handle: Handle) {
        {
            let mut table = self.table.lock();
            table.insert(name.clone(), handle);
        }

        let mut waiters = self.waiters.lock();

        for i in waiters.drain(..) {
            i.wake();
        }
    }

    pub async fn get<'a>(&'a self, name: &'a str) -> &'a Handle {
        struct HandleWaiter<'a> {
            name: &'a str,
            table: &'a HandleTable,
        }

        impl<'a> Future for HandleWaiter<'a> {
            type Output = &'a Handle;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let table = self.table.table.lock();

                if let Some(handle) = table.get(self.name) {
                    // fuck borrow checker. Idk how to explain it that handle is reference with
                    // lifetime of 'a, not of table...
                    //
                    // I don't know rust sorry
                    Poll::Ready(unsafe { &*(handle as *const _) })
                } else {
                    self.table.waiters.lock().push(cx.waker().clone());
                    Poll::Pending
                }
            }
        }

        HandleWaiter {
            name: name.as_ref(),
            table: self,
        }
        .await
    }
}
