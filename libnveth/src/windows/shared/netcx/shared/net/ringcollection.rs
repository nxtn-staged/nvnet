use crate::windows::shared::netcx::shared::net::ring::NET_RING;

c_type!(
    pub struct NET_RING_TYPE(usize);
);

impl NET_RING_TYPE {
    pub const NetRingTypePacket: usize = 0;
    pub const NetRingTypeFragment: usize = 1;
}

c_type!(
    pub struct NET_RING_COLLECTION {
        pub rings: [*mut NET_RING; NET_RING_TYPE::NetRingTypeFragment + 1],
    }
);

pub unsafe fn NetRingCollectionGetPacketRing(rings: *const NET_RING_COLLECTION) -> *mut NET_RING {
    (*rings).rings[NET_RING_TYPE::NetRingTypePacket]
}

pub unsafe fn NetRingCollectionGetFragmentRing(rings: *const NET_RING_COLLECTION) -> *mut NET_RING {
    (*rings).rings[NET_RING_TYPE::NetRingTypeFragment]
}
