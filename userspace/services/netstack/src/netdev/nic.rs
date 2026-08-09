use crate::bindings_NameServer::NameServer;
use alloc::vec::Vec;
use bindings_Nic::Nic as NicBindings;
use net::ethernet::Mac;
use rokio::port::Port;
use rtl::error::ErrorType;

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

    pub async fn send_packet(&self, data: &[u8]) -> Result<(), ErrorType> {
        self.nic
            .Send(data.try_into().map_err(|_| ErrorType::BufferTooBig)?)
            .await?;

        Ok(())
    }

    pub async fn mac(&self) -> Result<Mac, ErrorType> {
        let res = self.nic.Mac().await?;

        Mac::try_from(res.mac)
    }
}

include!(concat!(env!("OUT_DIR"), "/nic.rs"));
