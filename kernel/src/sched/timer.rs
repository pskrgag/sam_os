use super::ticks::SYSTEM_TICK;
use super::ticks::{SchedTicks, sched_ticks};
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use core::time::Duration;

struct Timer {
    cb: Box<dyn Fn() + Send>,
    dl: SchedTicks,
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

    pub fn set_timer(&mut self, dl: Duration, cb: Box<dyn Fn() + Send>) {
        let dl_ms = dl.as_millis() as u64;
        let tick_ms = SYSTEM_TICK.as_millis() as u64;
        // roundup(y, x) = x * round(y / x)
        let rounded = dl_ms.div_ceil(tick_ms) * tick_ms;

        let dl = sched_ticks() + rounded / tick_ms;

        self.insert(Timer { cb, dl });
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

pub fn set_timer(dl: Duration, cb: Box<dyn Fn() + Send>) {
    TIMER_QUEUE.lock().set_timer(dl, cb);
}

pub fn sched_tick() {
    TIMER_QUEUE.lock().on_sched_tick()
}
