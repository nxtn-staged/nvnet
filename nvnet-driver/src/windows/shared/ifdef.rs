// L154
c_type!(
    pub struct NET_IFTYPE(pub u16);
);

// L176
c_type!(
    pub enum NET_IF_CONNECTION_TYPE {
        NET_IF_CONNECTION_DEDICATED = 1,
    }
);

// L218
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
        MediaConnectStateConnected = 1,
        MediaConnectStateDisconnected = 2,
    }
);

c_type!(
    pub enum NET_IF_MEDIA_DUPLEX_STATE {
        MediaDuplexStateFull = 2,
    }
);

// L284
pub const IF_MAX_PHYS_ADDRESS_LENGTH: usize = 32;
