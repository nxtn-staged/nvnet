use crate::windows::shared::ws2def::ADDRESS_FAMILY;

c_type!(
    #[derive(Clone)]
    pub struct SOCKADDR_IN6 {
        pub family: ADDRESS_FAMILY,
        pub port: u16,
        pub flowinfo: u32,
        pub addr: [u8; 16],
        pub scope_id: u32,
    }
);

pub const IPV6_V6ONLY: u32 = 27;
