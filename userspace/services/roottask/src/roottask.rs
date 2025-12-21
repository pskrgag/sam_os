use super::bindings_NameServer as bindings;
use alloc::{collections::btree_map::BTreeMap, string::String};
use libc::{handle::Handle, port::Port};
use rtl::{error::ErrorType, locking::spinlock::Spinlock};

#[derive(Default)]
struct NameServer {
    table: BTreeMap<String, Handle>,
}

pub fn start(p: Port) {
    let mut server = bindings::NameServer::new(p, Spinlock::new(NameServer::default()))
        .register_handler(|t: bindings::RegisterTx, roottask| {
            let mut roottask = roottask.lock();

            if roottask.table.contains_key(&t.name) {
                return Err(ErrorType::AlreadyExists);
            }

            roottask.table.insert(t.name, t.handle);
            Ok(bindings::RegisterRx {})
        })
        .register_handler(|t: bindings::GetTx, roottask| {
            let roottask = roottask.lock();

            if let Some(h) = roottask.table.get(&t.name) {
                let h = h.clone_handle().unwrap();

                Ok(bindings::GetRx { handle: h })
            } else {
                Err(ErrorType::NotFound)
            }
        });

    println!("Starting nameserver...");
    server.run().unwrap();
}
