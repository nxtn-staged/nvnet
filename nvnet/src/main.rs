#![feature(default_free_fn)]
#![feature(duration_constants)]

mod bindings;
mod crypto;
mod device;
mod error;
mod ext;
mod ioctl;
mod windows;

use std::{
    convert::TryFrom,
    default::default,
    env,
    error::Error,
    fs::File,
    io::stdin,
    mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str, thread,
    time::Duration,
};

use serde::Deserialize;

use winapi::shared::{
    bcrypt::{BCRYPT_ECCKEY_BLOB, BCRYPT_ECDH_PRIVATE_GENERIC_MAGIC},
    ws2def::AF_INET6,
};

use crate::{crypto::ecdh::Ecdh, device::Device, ext::AsBytesExt, ioctl::*};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    #[serde(default = "Config::default_curve")]
    curve: Curve,
    #[serde(default = "Config::default_cipher")]
    cipher: Cipher,
    #[serde(default = "Config::default_hash")]
    hash: Hash,
    #[serde(default = "Config::default_dev")]
    dev: String,
    local: LocalEndPoint,
    remote: Vec<RemoteEndPoint>,
}

impl Config {
    fn default_curve() -> Curve {
        Curve::Curve25519
    }

    fn default_cipher() -> Cipher {
        Cipher::Aes128Gcm
    }

    fn default_hash() -> Hash {
        Hash::Sha256
    }

    fn default_dev() -> String {
        "NVEth0".into()
    }
}

#[derive(Deserialize)]
enum Curve {
    #[serde(rename = "curve25519")]
    Curve25519,
}

#[derive(Deserialize)]
enum Cipher {
    #[serde(rename = "aes-128-gcm")]
    Aes128Gcm,
    #[serde(rename = "aes-192-gcm")]
    Aes192Gcm,
    #[serde(rename = "aes-256-gcm")]
    Aes256Gcm,
}

#[derive(Deserialize)]
enum Hash {
    #[serde(rename = "sha-256")]
    Sha256,
    #[serde(rename = "sha-512")]
    Sha512,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct LocalEndPoint {
    endpoint: IpEndpoint,
    private_key: Option<Key>,
    public_key: Option<Key>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RemoteEndPoint {
    endpoint: IpEndpoint,
    addr: IpAddr,
    public_key: Option<Key>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
enum IpEndpoint {
    Scalar(SocketAddr),
    Mapping { addr: IpAddr, port: u16 },
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Key(Vec<u8>);

impl TryFrom<String> for Key {
    type Error = base64::DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
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

#[repr(C)]
struct BCryptEccKeyBlob<T> {
    header: BCRYPT_ECCKEY_BLOB,
    x: T,
    y: T,
    d: T,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    if let Some(cmd) = env::args().nth(1) {
        const KEY_SIZE: usize = 32;
        let mut key_blob = unsafe { mem::zeroed::<BCryptEccKeyBlob<[u8; KEY_SIZE]>>() };
        match cmd.as_str() {
            "eckv" => {
                let key = Ecdh::new()?;
                key.export_private_key_slice(key_blob.as_mut_bytes())?;
                let key_bytes = &key_blob.d;
                println!("{}", base64::encode(key_bytes));
                return Ok(());
            }
            "eckp" => {
                let mut key_str = String::new();
                stdin().read_line(&mut key_str)?;
                let key_str = key_str.trim_end();
                let key_bytes = &mut key_blob.d;
                base64::decode_config_slice(key_str, base64::STANDARD, key_bytes)?;
                key_blob.header.dwMagic = BCRYPT_ECDH_PRIVATE_GENERIC_MAGIC;
                key_blob.header.cbKey = KEY_SIZE as _;
                let key = Ecdh::import(key_blob.as_bytes())?;
                key.export_public_key_slice(key_blob.as_mut_bytes())?;
                let pub_key_bytes = &key_blob.x;
                println!("{}", base64::encode(pub_key_bytes));
                return Ok(());
            }
            _ => {}
        }
    }

    let path = env::args().nth(1).unwrap_or("nvnet.yml".into());
    let file = File::open(path)?;

    let config: Config = serde_yaml::from_reader(file)?;

    let device = Device::open(format!(r"\\.\Global\{}", config.dev))?;

    let to_raw_socket_addr = |endpoint: &IpEndpoint| {
        let (addr, port) = match endpoint {
            IpEndpoint::Scalar(addr) => match addr {
                SocketAddr::V4(v4) => (v4.ip().to_ipv6_mapped().octets(), v4.port()),
                SocketAddr::V6(v6) => (v6.ip().octets(), v6.port()),
            },
            IpEndpoint::Mapping { addr, port } => match addr {
                IpAddr::V4(v4) => (v4.to_ipv6_mapped().octets(), *port),
                IpAddr::V6(v6) => (v6.octets(), *port),
            },
        };
        SOCKADDR_IN6 {
            sin6_family: AF_INET6 as _,
            sin6_port: port.to_be(),
            sin6_addr: addr,
            ..default()
        }
    };

    let local_socket_addr = to_raw_socket_addr(&config.local.endpoint);
    device.control_in_ref(IOCTL_VETH_SET_LOCAL_ADDR, &local_socket_addr)?;

    for remote in &config.remote {
        let remote_socket_addr = to_raw_socket_addr(&remote.endpoint);
        device.control_in_ref(IOCTL_VETH_ADD_REMOTE_PEER, &remote_socket_addr)?;
    }

    device.control_in(IOCTL_VETH_SET_CONNECT_STATE, true)?;

    thread::sleep(Duration::MAX);
    Ok(())
}

macro_rules! assert_matches {
    ($expr:expr, [$($name:ident),*]) => {
        assert_matches!($expr, [$($name),*] => ($($name),*))
    };
    ($expr:expr, $($path:ident)::+($($name:ident),*)) => {
        assert_matches!($expr, $($path)::+($($name),*) => ($($name),*))
    };
    ($expr:expr, $path:path { $($name:ident),* }) => {
        assert_matches!($expr, $path { $($name),* } => ($($name),*))
    };
    ($expr:expr, $pat:pat) => {
        assert!(matches!($expr, $pat))
    };
    ($expr:expr, $pat:pat => $ret:expr) => {
        match $expr {
            $pat => $ret,
            _ => panic!(),
        }
    };
}

#[test]
fn config_complete() -> Result<(), serde_yaml::Error> {
    let s = r"
curve: curve25519
cipher: aes-256-gcm
hash: sha-512
dev: NVEth1

local:
  endpoint: '[::]:5001'
  private-key: gFDRW4oyzSBH9ig8JHs4f9MA5xc6zZDOj2Z/hDB3gEM=
  public-key: 1SWFFZlt8UBfD2BCxd7YgM/oqc31I2evWsAOAygtbBM=

remote:
  - endpoint:
      addr: 169.254.123.180
      port: 5001
    addr: 0.0.0.0
    public-key: x9FYGi0CJlt810zXnokkoZ2alQd6gksA2fbJil2leE8=
";

    let config: Config = serde_yaml::from_str(s)?;

    assert_matches!(config.curve, Curve::Curve25519);
    assert_matches!(config.cipher, Cipher::Aes256Gcm);
    assert_matches!(config.hash, Hash::Sha512);
    assert_eq!(config.dev.as_str(), "NVEth1");

    let addr = assert_matches!(&config.local.endpoint, IpEndpoint::Scalar(addr));
    let v6 = assert_matches!(addr, SocketAddr::V6(v6));
    assert_eq!(*v6.ip(), Ipv6Addr::UNSPECIFIED);
    assert_eq!(v6.port(), 5001);

    let key = config.local.private_key.as_ref().unwrap();
    assert_eq!(
        base64::encode(key),
        String::from("gFDRW4oyzSBH9ig8JHs4f9MA5xc6zZDOj2Z/hDB3gEM="),
    );
    let key = config.local.public_key.as_ref().unwrap();
    assert_eq!(
        base64::encode(key),
        String::from("1SWFFZlt8UBfD2BCxd7YgM/oqc31I2evWsAOAygtbBM="),
    );

    let peer = assert_matches!(&config.remote.as_slice(), [peer]);
    let (addr, port) = assert_matches!(&peer.endpoint, IpEndpoint::Mapping { addr, port });
    assert_eq!(*addr, Ipv4Addr::new(169, 254, 123, 180));
    assert_eq!(*port, 5001);

    let key = peer.public_key.as_ref().unwrap();
    assert_eq!(
        base64::encode(key),
        String::from("x9FYGi0CJlt810zXnokkoZ2alQd6gksA2fbJil2leE8="),
    );

    Ok(())
}

macro_rules! assert_variant_eq {
    ($lhs:expr, $rhs:expr) => {
        assert_eq!(mem::discriminant(&$lhs), mem::discriminant(&$rhs))
    };
}

#[test]
fn config_minimal() -> Result<(), serde_yaml::Error> {
    let s = r"
local:
  endpoint:
    addr: 0.0.0.0
    port: 5001

remote:
  - endpoint:
      addr: 169.254.123.180
      port: 5001
    addr: 0.0.0.0
";

    let config: Config = serde_yaml::from_str(s)?;

    assert_variant_eq!(config.curve, Config::default_curve());
    assert_variant_eq!(config.cipher, Config::default_cipher());
    assert_variant_eq!(config.hash, Config::default_hash());
    assert_eq!(config.dev, Config::default_dev());

    assert_matches!(config.local.private_key, None);
    assert_matches!(config.local.public_key, None);

    let peer = assert_matches!(&config.remote.as_slice(), [peer]);
    let (addr, port) = assert_matches!(&peer.endpoint, IpEndpoint::Mapping { addr, port });
    assert_eq!(*addr, Ipv4Addr::new(169, 254, 123, 180));
    assert_eq!(*port, 5001);

    assert_matches!(peer.public_key, None);

    Ok(())
}
