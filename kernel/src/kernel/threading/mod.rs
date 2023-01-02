pub mod thread;
pub mod thread_ep;
pub mod thread_table;

use alloc::sync::Arc;
use qrwlock::qrwlock::RwLock;
use thread::Thread;

pub type ThreadRef = Arc<RwLock<Thread>>;
