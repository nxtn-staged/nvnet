use crate::windows::km::xfilter::{ETH_IS_BROADCAST, ETH_IS_MULTICAST};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MacAddr([u8; 6]);

impl MacAddr {
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        Self([a, b, c, d, e, f])
    }

    pub fn bytes(&self) -> &[u8; 6] {
        &self.0
    }

    pub fn is_broadcast(&self) -> bool {
        ETH_IS_BROADCAST(&self.0)
    }

    pub fn is_multicast(&self) -> bool {
        ETH_IS_MULTICAST(&self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Addr([u8; 4]);

impl Ipv4Addr {
    pub fn from_ipv6_mapped(ipv6: &[u8; 16]) -> Option<Self> {
        match ipv6 {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, a, b, c, d] => Some(Self([*a, *b, *c, *d])),
            _ => None,
        }
    }
}

#[repr(C)]
pub struct EthFrameHeader {
    dst_addr: MacAddr,
    src_addr: MacAddr,
    eth_type: [u8; 2],
}

impl EthFrameHeader {
    pub fn dst_addr(&self) -> &MacAddr {
        &self.dst_addr
    }

    pub fn src_addr(&self) -> &MacAddr {
        &self.src_addr
    }

    pub fn is_arp(&self) -> bool {
        self.eth_type[0] == 0x08 && self.eth_type[1] == 0x06
    }
}

#[repr(C)]
pub struct ArpPacket {
    hardware_type: [u8; 2],
    protocol_type: [u8; 2],
    hardware_len: u8,
    protocol_len: u8,
    operation: [u8; 2],
    src_hardware_addr: MacAddr,
    src_protocol_addr: Ipv4Addr,
    dst_hardware_addr: MacAddr,
    dst_protocol_addr: Ipv4Addr,
}

impl ArpPacket {
    pub fn is_request(&self) -> bool {
        self.operation[0] == 0x00 && self.operation[1] == 0x01
    }

    pub fn is_reply(&self) -> bool {
        self.operation[0] == 0x00 && self.operation[1] == 0x10
    }

    pub fn src_mac(&self) -> MacAddr {
        self.src_hardware_addr.clone()
    }

    pub fn dst_eq_ipv4(&self, addr: &Ipv4Addr) -> bool {
        self.dst_protocol_addr == *addr
    }
}
