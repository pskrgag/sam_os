use crate::bindings_NameServer::NameServer;
use bindings_Nic::Nic as NicBindings;
use rokio::port::Port;
use rtl::error::ErrorType;
use alloc::vec::Vec;

pub struct Nic {
    nic: NicBindings,
}

impl Nic {
    pub async fn new(ns: &NameServer) -> Result<Self, ErrorType> {
        let nic = ns.Get("nic".try_into().unwrap()).await.unwrap();

        Ok(Self {
            nic: NicBindings::new(unsafe { Port::new(nic.handle) }),
        })
    }

    pub async fn read_packet(&self) -> Result<Vec<u8>, ErrorType> {
        let res = self.nic.Receive().await?;

        Ok(Vec::from_iter(res.data.into_iter()))
    }
}

include!(concat!(env!("OUT_DIR"), "/nic.rs"));
