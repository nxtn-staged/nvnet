c_type!(
    pub struct NET_IFTYPE(pub u16);
);

c_type!(
    pub enum NET_IF_CONNECTION_TYPE {
        NET_IF_CONNECTION_DEDICATED = 1,
    }
);

c_type!(
    pub enum NET_IF_ACCESS_TYPE {
        NET_IF_ACCESS_BROADCAST = 2,
    }
);

c_type!(
    pub enum NET_IF_DIRECTION_TYPE {
        NET_IF_DIRECTION_SENDRECEIVE = 0,
    }
);

c_type!(
    pub enum NET_IF_MEDIA_CONNECT_STATE {
        MediaConnectStateUnknown = 0,
        MediaConnectStateConnected = 1,
        MediaConnectStateDisconnected = 2,
    }
);

c_type!(
    pub enum NET_IF_MEDIA_DUPLEX_STATE {
        MediaDuplexStateUnknown = 0,
        MediaDuplexStateFull = 2,
    }
);

pub const IF_MAX_PHYS_ADDRESS_LENGTH: usize = 32;

c_type!(
    pub struct IF_PHYSICAL_ADDRESS {
        pub length: u16,
        pub address: [u8; IF_MAX_PHYS_ADDRESS_LENGTH],
    }
);
