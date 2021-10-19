#![feature(entry_insert)]
#![feature(never_type)]

use std::{
    collections::HashMap,
    error::Error,
    mem,
    net::{Ipv4Addr, UdpSocket},
};

const MAX_ETH_MTU_SIZE: usize = 1500;
const MAX_IP_HEADER_SIZE: usize = 60;
const MAX_UDP_HEADER_SIZE: usize = 8;

const MAX_FRAME_SIZE: usize = MAX_ETH_MTU_SIZE - MAX_IP_HEADER_SIZE - MAX_UDP_HEADER_SIZE;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MacAddr([u8; 6]);

impl MacAddr {
    fn is_broadcast(&self) -> bool {
        self.0 == [0xff; 6]
    }

    fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }
}

#[repr(C)]
struct EthHeader {
    dst_addr: MacAddr,
    src_addr: MacAddr,
    eth_type: [u8; 2],
}

struct MainError(Box<dyn Error>);

impl std::fmt::Debug for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl<E: Error + 'static> From<E> for MainError {
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

fn main() -> Result<!, MainError> {
    let mut clients = HashMap::new();
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 23333))?;
    let mut buf = vec![0; MAX_FRAME_SIZE];
    loop {
        let (received, addr) = match socket.recv_from(&mut buf) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        if received < mem::size_of::<EthHeader>() {
            continue;
        }
        let buf = &buf[..received];
        let eth = unsafe { &*buf.as_ptr().cast::<EthHeader>() };
        let src_addr = &eth.src_addr;
        clients.entry(*src_addr).insert(addr);
        let dst_addr = &eth.dst_addr;
        if dst_addr.is_multicast() {
            if dst_addr.is_broadcast() {
                for (mac_addr, socket_addr) in &clients {
                    if mac_addr == src_addr {
                        continue;
                    }
                    if let Err(err) = socket.send_to(buf, socket_addr) {
                        eprintln!("{}", err);
                    }
                }
            }
        } else {
            if let Some(socket_addr) = clients.get(dst_addr) {
                if let Err(err) = socket.send_to(buf, socket_addr) {
                    eprintln!("{}", err);
                }
            }
        }
    }
}
