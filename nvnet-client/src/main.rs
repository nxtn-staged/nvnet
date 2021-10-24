#![feature(default_free_fn)]

mod bindings;
mod device;

use crate::device::Device;
use serde::Deserialize;
use std::{
    default::default,
    env,
    error::Error,
    fs::File,
    net::{IpAddr, Ipv6Addr},
};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "Config::default_dev")]
    dev: String,
    local: LocalEndpoint,
    remote: RemoteEndpoint,
}

impl Config {
    fn default_dev() -> String {
        "NVNet0".into()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LocalEndpoint {
    #[serde(default = "LocalEndpoint::default_endpoint")]
    endpoint: Endpoint,
}

impl LocalEndpoint {
    fn default_endpoint() -> Endpoint {
        Endpoint {
            addr: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            port: 0,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteEndpoint {
    endpoint: Endpoint,
    secret_key: Option<Key>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Endpoint {
    addr: IpAddr,
    port: u16,
}

impl Endpoint {
    fn to_raw_addr(&self) -> SOCKADDR_IN6 {
        let addr = match self.addr {
            IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            IpAddr::V6(v6) => v6,
        };
        let addr = addr.octets();
        let port = self.port;
        SOCKADDR_IN6 {
            sin6_family: 23, // TODO: AF_INET6
            sin6_port: port.to_be(),
            sin6_addr: addr,
            ..default()
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "&str")]
struct Key(Vec<u8>);

impl TryFrom<&str> for Key {
    type Error = base64::DecodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(base64::decode(value)?))
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        &self.0
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

fn main() -> Result<(), MainError> {
    let path = env::args_os().nth(1).unwrap_or("nvnet.yml".into());
    let file = File::open(path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    let device = Device::open(format!(r"\\.\Global\{}", config.dev))?;
    let local_addr = config.local.endpoint.to_raw_addr();
    device.control_in(nvnet_shared::IOCTL_VNET_SET_LOCAL_ENDPOINT, local_addr)?;
    let remote_addr = config.remote.endpoint.to_raw_addr();
    device.control_in(nvnet_shared::IOCTL_VNET_SET_REMOTE_ENDPOINT, remote_addr)?;
    if let Some(secret_key) = config.remote.secret_key {
        let secret_key = secret_key.as_ref();
        device.control_in_ref(nvnet_shared::IOCTL_VNET_SET_REMOTE_SECRET_KEY, secret_key)?;
    }
    device.control_in(nvnet_shared::IOCTL_VNET_SET_CONNECT_STATE, true)?;
    Ok(())
}
