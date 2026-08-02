/// ARP handler
use alloc::collections::BTreeMap;
use net::eth::mac::Mac;
use net::ip::v4::IPv4;

#[derive(Default)]
pub struct ArpCache {
    cache: BTreeMap<IPv4, Mac>,
}

impl ArpCache {
    pub fn insert(&mut self, ip: IPv4, mac: Mac) {
        self.cache.insert(ip, mac);
    }
}
