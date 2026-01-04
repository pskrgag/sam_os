use super::bindings_NameServer as bindings;
use super::handle_table::HandleTable;
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use bindings::NameServerRequest;
use rokio::port::Port;

pub async fn start(p: Port) {
    println!("Starting nameserver...");

    let ns = Arc::new(HandleTable::new());

    bindings::NameServer::for_each(p, move |request| {
        let ns = ns.clone();

        async move {
            match request {
                NameServerRequest::Get { value, responder } => {
                    responder.reply(ns.get(value.name.as_str()).await)?;
                }
                NameServerRequest::Register { value, responder } => {
                    ns.insert(value.name.as_str().to_owned(), value.handle);
                    responder.reply()?;
                }
            };

            Ok(())
        }
    })
    .await
    .unwrap();
}
