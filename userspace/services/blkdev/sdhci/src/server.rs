use super::bindings_NameServer::NameServer;
use super::sdhci::Sdhci;
use alloc::vec::Vec;
use bindings_BlkDev::{BlkDev, ReadBlockRx, ReadBlockTx};
use libc::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub fn start_server(sdhci: Sdhci, ns: &NameServer) -> Result<(), ErrorType> {
    let port = Port::create()?;

    ns.Register("blkdev", port.handle())
        .expect("Failed to register handle in nameserver");

    BlkDev::new(port, Spinlock::new(sdhci))
        .register_handler(|req: ReadBlockTx, card| {
            let mut card = card.lock();
            let mut data = Vec::with_capacity(card.block_size());

            data.resize(card.block_size(), 0);

            card.read_block(req.blockIdx, data.as_mut_slice())?;
            Ok(ReadBlockRx { data })
        })
        .run()
}

include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
