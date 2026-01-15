use super::bindings_NameServer::NameServer;
use crate::cards::Card;
use alloc::boxed::Box;
use alloc::sync::Arc;
use bindings_BlkDev::{BlkDev, BlkDevRequest, BlockInfo};
use rokio::port::Port;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub async fn start_server(sdhci: Box<dyn Card>, ns: NameServer) -> Result<(), ErrorType> {
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
                    let data = card.read_block(value.blockIdx)?;

                    responder.reply(data.into_iter().collect())?;
                }
                BlkDevRequest::GetInfo { responder, .. } => {
                    let card = sdhci.lock();

                    responder.reply(BlockInfo {
                        blockSize: card.block_size(),
                        blockCount: card.device_size() / card.block_size() as usize,
                    })?;
                }
            }
            Ok(())
        }
    })
    .await
}

include!(concat!(env!("OUT_DIR"), "/blkdev.rs"));
