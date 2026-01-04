use super::bindings_NameServer as bindings;
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::{collections::btree_map::BTreeMap, string::String};
use bindings::{GetRx, NameServerRequest, RegisterRx};
use libc::handle::Handle;
use rokio::port::Port;
use rtl::{error::ErrorType, locking::spinlock::Spinlock};

#[derive(Default)]
struct NameServer {
    table: BTreeMap<String, Handle>,
}

pub async fn start(p: Port) {
    println!("Starting nameserver...");

    let ns = Arc::new(Spinlock::new(NameServer::default()));

    bindings::NameServer::for_each(p, |request| {
        let ns = ns.clone();

        async move {
            match request {
                NameServerRequest::Get { value, responder } => {
                    responder.reply(GetRx {
                        handle: ns
                            .lock()
                            .table
                            .get(value.name.as_str())
                            .ok_or(ErrorType::NotFound)?
                            .clone_handle()?,
                    })?;
                }
                NameServerRequest::Register { value, responder } => {
                    ns.lock()
                        .table
                        .insert(value.name.as_str().to_owned(), value.handle);
                    responder.reply(RegisterRx {})?;
                }
            };

            Ok(())
        }
    })
    .await
    .unwrap();
}
