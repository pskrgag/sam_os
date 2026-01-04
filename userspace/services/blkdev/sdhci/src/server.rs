use super::bindings_NameServer::NameServer;
use super::sdhci::Sdhci;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bindings_BlkDev::{BlkDev, BlkDevRequest, BlockInfo};
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
                    responder.reply(data.into_iter().collect())?;
                }
                BlkDevRequest::GetInfo { responder, .. } => {
                    let card = sdhci.lock();

                    responder.reply(BlockInfo {
                        blockSize: card.block_size(),
                        blockCount: 0,
                    })?;
                }
            }
            Ok(())
        }
    })
    .await
}

include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
