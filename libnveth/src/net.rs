use crate::windows::km::xfilter::{ETH_IS_BROADCAST, ETH_IS_MULTICAST};

#[repr(transparent)]
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

pub enum IpAddr {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

impl IpAddr {
    pub fn from_ipv6(ipv6: &[u8; 16]) -> Self {
        match ipv6 {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, a, b, c, d] => {
                Self::Ipv4(Ipv4Addr([*a, *b, *c, *d]))
            }
            _ => IpAddr::Ipv6(Ipv6Addr(*ipv6)),
        }
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Ipv4Addr([u8; 4]);

impl Ipv4Addr {
    pub fn is_unspecified(&self) -> bool {
        self.0 == [0; 4]
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Ipv6Addr([u8; 16]);

#[repr(C)]
pub struct EthHeader {
    dst_addr: MacAddr,
    src_addr: MacAddr,
    eth_type: [u8; 2],
}

impl EthHeader {
    pub fn dst(&self) -> &MacAddr {
        &self.dst_addr
    }

    pub fn is_arp(&self) -> bool {
        self.eth_type[0] == 0x08 && self.eth_type[1] == 0x06
    }

    pub fn is_ipv6(&self) -> bool {
        self.eth_type[0] == 0x86 && self.eth_type[1] == 0xdd
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
    pub fn is_eth(&self) -> bool {
        self.hardware_type[0] == 0x00 && self.hardware_type[1] == 0x01 && self.hardware_len == 6
    }

    pub fn is_ipv4(&self) -> bool {
        self.protocol_type[0] == 0x08 && self.protocol_type[1] == 0x00 && self.protocol_len == 4
    }

    pub fn src_mac(&self) -> &MacAddr {
        &self.src_hardware_addr
    }

    pub fn src_ipv4(&self) -> &Ipv4Addr {
        &self.src_protocol_addr
    }
}

#[repr(C)]
pub struct Layer2ArpPacket {
    pub eth: EthHeader,
    pub arp: ArpPacket,
}

#[repr(C)]
pub struct Ipv6Header {
    unused: [u8; 4],
    payload_len: u16,
    next_header: u8,
    hop_limit: u8,
    src_addr: Ipv6Addr,
    dst_addr: Ipv6Addr,
}

impl Ipv6Header {
    pub fn is_icmpv6(&self) -> bool {
        self.next_header == 0x3a
    }
}

#[repr(C)]
pub struct Icmpv6Header {
    r#type: u8,
    code: u8,
    checksum: u16,
}

impl Icmpv6Header {
    pub fn is_neighbor_solicitation(&self) -> bool {
        self.r#type == 135 && self.code == 0
    }

    pub fn is_neighbor_advertisement(&self) -> bool {
        self.r#type == 136 && self.code == 0
    }
}

#[repr(C)]
pub struct L2Icmpv6Header {
    pub eth: EthHeader,
    pub ipv6: Ipv6Header,
    pub icmpv6: Icmpv6Header,
}

#[repr(C)]
pub struct Icmpv6NsHeader {
    header: Icmpv6Header,
    unused: [u8; 4],
    target_addr: Ipv6Addr,
    opt_type: u8,
    opt_len: u8,
    source_mac_addr: MacAddr,
}

impl Icmpv6NsHeader {
    pub fn source_mac(&self) -> Option<&MacAddr> {
        if self.opt_type == 1 && self.opt_len == 1 {
            Some(&self.source_mac_addr)
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct L2Icmpv6NsHeader {
    pub eth: EthHeader,
    pub ipv6: Ipv6Header,
    pub icmpv6_ns: Icmpv6NsHeader,
}

#[repr(C)]
pub struct Icmpv6NaHeader {
    header: Icmpv6Header,
    unused: [u8; 4],
    target_addr: Ipv6Addr,
    opt_type: u8,
    opt_len: u8,
    target_mac_addr: MacAddr,
}

impl Icmpv6NaHeader {
    pub fn target_mac(&self) -> Option<&MacAddr> {
        if self.opt_type == 2 && self.opt_len == 1 {
            Some(&self.target_mac_addr)
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct L2Icmpv6NaHeader {
    pub eth: EthHeader,
    pub ipv6: Ipv6Header,
    pub icmpv6_na: Icmpv6NaHeader,
}
