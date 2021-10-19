// L81
c_type!(
    pub struct ADDRESS_FAMILY(pub u16);
);

// L121
pub const AF_INET6: ADDRESS_FAMILY = ADDRESS_FAMILY(23);

pub const SOCK_DGRAM: u16 = 2;

// L240
c_type!(
    pub struct SOCKADDR;
);

// L438
c_type!(
    pub enum IPPROTO {
        IPPROTO_UDP = 17,
        IPPROTO_IPV6 = 41,
    }
);

// L726
c_type!(
    pub struct WSACMSGHDR;
);
