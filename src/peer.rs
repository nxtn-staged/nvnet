use crate::{net::MacAddr, windows::prelude as win};

pub struct Peer {
    pub socket_addr: win::SOCKADDR_IN6,
    pub mac_addr: Option<MacAddr>,
}

impl Peer {
    pub fn new(addr: win::SOCKADDR_IN6) -> Self {
        Self {
            socket_addr: addr,
            mac_addr: None,
        }
    }
}
