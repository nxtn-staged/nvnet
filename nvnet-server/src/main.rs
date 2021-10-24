#![feature(entry_insert)]
#![feature(never_type)]

use crossbeam_channel::{self, Receiver};
use std::{
    collections::HashMap,
    error::Error,
    io, mem,
    net::{IpAddr, Ipv6Addr, SocketAddr, UdpSocket},
    os::unix::prelude::FromRawFd,
    sync::Arc,
    thread,
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

struct Packet {
    buf: Arc<Vec<u8>>,
    len: usize,
    addr: SocketAddr,
}

fn main() -> Result<!, MainError> {
    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 23333));
    let (channel_tx, channel_rx) = crossbeam_channel::bounded(1024);
    thread::spawn(move || start_tx(addr, channel_rx));
    let mut clients = HashMap::new();
    let socket = bind(addr)?;
    loop {
        let mut buf = vec![0; MAX_FRAME_SIZE];
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
        let frame = &buf[..received];
        let eth = unsafe { &*frame.as_ptr().cast::<EthHeader>() };
        let src_addr = &eth.src_addr;
        clients.entry(*src_addr).insert(addr);
        let dst_addr = &eth.dst_addr;
        if dst_addr.is_multicast() {
            if dst_addr.is_broadcast() {
                let buf = Arc::new(buf);
                for (mac_addr, socket_addr) in &clients {
                    if mac_addr == src_addr {
                        continue;
                    }
                    channel_tx.send(Packet {
                        buf: buf.clone(),
                        len: received,
                        addr: *socket_addr,
                    })?;
                }
            }
        } else {
            if let Some(socket_addr) = clients.get(dst_addr) {
                channel_tx.send(Packet {
                    buf: Arc::new(buf),
                    len: received,
                    addr: *socket_addr,
                })?;
            }
        }
    }
}

fn bind(addr: SocketAddr) -> io::Result<UdpSocket> {
    fn ok(i: i32) -> io::Result<i32> {
        if i == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(i)
        }
    }

    let socket = unsafe { ok(libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, 0))? };
    let val = 1;
    unsafe {
        ok(libc::setsockopt(
            socket,
            libc::SOL_SOCKET,
            libc::SO_REUSEPORT,
            &val as *const _ as *const _,
            mem::size_of_val(&val) as u32,
        ))?
    };
    let val = 0;
    unsafe {
        ok(libc::setsockopt(
            socket,
            libc::SOL_IPV6,
            libc::IPV6_V6ONLY,
            &val as *const _ as *const _,
            mem::size_of_val(&val) as u32,
        ))?
    };
    let raw_addr = libc::sockaddr_in6 {
        sin6_family: libc::AF_INET6 as u16,
        sin6_port: addr.port().to_be(),
        sin6_addr: libc::in6_addr {
            s6_addr: {
                let addr = match addr.ip() {
                    IpAddr::V4(v4) => v4.to_ipv6_mapped(),
                    IpAddr::V6(v6) => v6,
                };
                addr.octets()
            },
        },
        ..unsafe { mem::zeroed() }
    };
    unsafe {
        ok(libc::bind(
            socket,
            &raw_addr as *const _ as *const _,
            mem::size_of_val(&raw_addr) as u32,
        ))?
    };
    unsafe { Ok(UdpSocket::from_raw_fd(socket)) }
}

fn start_tx(addr: SocketAddr, channel_rx: Receiver<Packet>) -> io::Result<!> {
    let socket = bind(addr)?;
    for packet in channel_rx {
        if let Err(err) = socket.send_to(&packet.buf[..packet.len], packet.addr) {
            eprintln!("{}", err);
        }
    }
    unreachable!();
}
