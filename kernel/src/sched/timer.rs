use super::ticks::SYSTEM_TICK;
use super::ticks::{sched_ticks, SchedTicks};
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;

struct Timer {
    cb: Box<dyn Fn() + Send>,
    dl: SchedTicks,
    handle: TimerHandle,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.dl == other.dl
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.dl.partial_cmp(&other.dl)
    }
}

pub struct TimerQueue {
    queue: LinkedList<Timer>,
}

static TIMER_QUEUE: Spinlock<TimerQueue> = Spinlock::new(TimerQueue::new());

pub struct TimerHandle(u64);

impl TimerHandle {
    fn allocate() -> Self {
        static TICKET: AtomicU64 = AtomicU64::new(0);

        Self(TICKET.fetch_add(1, Ordering::Relaxed))
    }

    pub fn cancel(self) {
        let mut queue = TIMER_QUEUE.lock_irqsave();

        queue.cancel(self);
    }
}

impl TimerQueue {
    const fn new() -> Self {
        Self {
            queue: LinkedList::new(),
        }
    }

    fn insert(&mut self, mut t: Timer) {
        let mut cursor = self.queue.cursor_front_mut();

        while let Some(cur) = cursor.current()
            && cur < &mut t
        {
            cursor.move_next();
        }

        cursor.insert_after(t);
    }

    fn cancel(&mut self, handle: TimerHandle) {
        let mut cursor = self.queue.cursor_front_mut();

        while let Some(cur) = cursor.current()
            && cur.handle.0 != handle.0
        {
            cursor.move_next();
        }

        cursor.remove_current();
    }

    pub fn set_timer(&mut self, dl: Duration, cb: Box<dyn Fn() + Send>) -> TimerHandle {
        let dl_ms = dl.as_millis() as u64;
        let tick_ms = SYSTEM_TICK.as_millis() as u64;
        let rounded = dl_ms.div_ceil(tick_ms) * tick_ms;

        let dl = sched_ticks() + rounded / tick_ms;
        let handle = TimerHandle::allocate();

        self.insert(Timer {
            cb,
            dl,
            handle: TimerHandle(handle.0),
        });

        handle
    }

    pub fn on_sched_tick(&mut self) {
        let current_tick = sched_ticks();
        let mut cursor = self.queue.cursor_front_mut();

        while let Some(cur) = cursor.current()
            && cur.dl == current_tick
        {
            // SAFETY: cursor points to element because of while check
            let cur = unsafe { cursor.remove_current().unwrap_unchecked() };

            (cur.cb)();
        }
    }
}

pub fn time_since_start() -> Duration {
    use crate::drivers::timer::SystemTimer;

    crate::arch::timer::SYSTEM_TIMER.since_start()
}

pub fn set_timer(dl: Duration, cb: Box<dyn Fn() + Send>) -> TimerHandle {
    TIMER_QUEUE.lock_irqsave().set_timer(dl, cb)
}

pub fn sched_tick() {
    super::current().tick();
    TIMER_QUEUE.lock_irqsave().on_sched_tick()
}
