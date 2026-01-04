use super::bindings_NameServer::NameServer;
use super::sdhci::Sdhci;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bindings_BlkDev::{BlkDev, BlkDevRequest, ReadBlockRx, ReadBlockTx};
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub async fn start_server(sdhci: Sdhci, ns: &NameServer) -> Result<(), ErrorType> {
    let port = Port::create()?;
    let sdhci = Arc::new(Spinlock::new(sdhci));

    ns.Register("blkdev".try_into().unwrap(), port.handle())
        .await
        .expect("Failed to register handle in nameserver");

    BlkDev::for_each(port, move |req| {
        let sdhci = sdhci.clone();

        async move {
            match req {
                BlkDevRequest::ReadBlock { value, responder } => {
                    let mut card = sdhci.lock();
                    let mut data = Vec::with_capacity(card.block_size());

                    data.resize(card.block_size(), 0);

                    card.read_block(value.blockIdx, data.as_mut_slice())?;
                    responder.reply(ReadBlockRx { data: data.into_iter().collect() })?;
                }
            }
            Ok(())
        }
    })
    .await
}

include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
