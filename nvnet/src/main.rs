#![feature(default_free_fn)]
#![feature(duration_constants)]

mod device;
mod error;
mod ext;
mod ioctl;
mod windows;

use std::{
    default::default,
    env,
    error::Error,
    fs::File,
    net::{IpAddr, Ipv6Addr},
    thread,
    time::Duration,
};

use serde::Deserialize;
use winapi::shared::ws2def::AF_INET6;

use crate::{device::Device, ioctl::*};

#[derive(Deserialize)]
struct LocalEndPoint {
    addr: IpAddr,
    port: u16,
}

#[derive(Deserialize)]
struct RemoteEndPoint {
    peer: IpAddr,
    addr: IpAddr,
    port: u16,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default = "Config::default_dev")]
    dev: String,
    local: LocalEndPoint,
    remote: Vec<RemoteEndPoint>,
}

impl Config {
    fn default_dev() -> String {
        "NVEth0".into()
    }
}

#[repr(C)]
#[derive(Default)]
struct SOCKADDR_IN6 {
    sin6_family: u16,
    sin6_port: u16,
    sin6_flowinfo: u32,
    sin6_addr: [u8; 16],
    sin6_scope_id: u32,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).unwrap_or("nvnet.yml".into());
    let file = File::open(path)?;

    let config: Config = serde_yaml::from_reader(file)?;

    let device = Device::open(format!(r"\\.\Global\{}", config.dev))?;

    let local_port = config.local.port;
    // let local_addr = match config.local.addr {
    //     IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
    //     IpAddr::V6(ipv6) => ipv6,
    // };
    let local_socket_addr = SOCKADDR_IN6 {
        sin6_family: AF_INET6 as _,
        sin6_port: local_port.to_be(),
        sin6_addr: Ipv6Addr::UNSPECIFIED.octets(),
        ..default()
    };
    device.control_in_ref(IOCTL_VETH_SET_LOCAL_ADDR, &local_socket_addr)?;

    let remote = &config.remote[0];
    let remote_port = remote.port;
    let remote_addr = match remote.peer {
        IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
        IpAddr::V6(ipv6) => ipv6,
    };
    let remote_socket_addr = SOCKADDR_IN6 {
        sin6_family: AF_INET6 as _,
        sin6_port: remote_port.to_be(),
        sin6_addr: remote_addr.octets(),
        ..default()
    };
    device.control_in_ref(IOCTL_VETH_ADD_REMOTE_PEER, &remote_socket_addr)?;

    device.control_in(IOCTL_VETH_SET_CONNECT_STATE, true)?;

    thread::sleep(Duration::MAX);
    Ok(())
}
