use core::default::default;

use crate::{
    net::{IpAddr, MacAddr},
    os::sync::RwLock,
    windows::prelude as win,
};

pub struct Peer {
    pub socket_addr: win::SOCKADDR_IN6,
    pub mac_addr: RwLock<Option<MacAddr>>,
    pub ip_addr: IpAddr,
}

impl Peer {
    pub fn new(addr: win::SOCKADDR_IN6) -> Self {
        Self {
            ip_addr: IpAddr::from_ipv6(&addr.addr),
            socket_addr: addr,
            mac_addr: default(),
        }
    }
}
