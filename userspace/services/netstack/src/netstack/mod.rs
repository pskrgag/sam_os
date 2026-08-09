mod inet;
mod netstack;
pub mod server;

pub use netstack::{NetStack, PacketDecision};

pub fn init() {
    inet::init();
}

pub use server::serve;
