use core::ffi::c_void;

use crate::windows::shared::netcx::shared::net::{fragment::NET_FRAGMENT, packet::NET_PACKET};

c_type!(
    pub struct NET_RING {
        os_reserved1: u16,
        element_stride: u16,
        number_of_elements: u32,
        element_index_mask: u32,
        pub end_index: u32,
        u: NET_RING_u,
        pub begin_index: u32,
        pub next_index: u32,
        scratch: *mut c_void,
        buffer: [u8; 1],
    }
);

c_type!(
    union NET_RING_u {
        os_reserved0: u32,
        os_reserved2: [*mut c_void; 4],
    }
);

unsafe fn NetRingGetElementAtIndex(ring: *mut NET_RING, index: u32) -> *mut c_void {
    assert!(index < (*ring).number_of_elements);
    (*ring)
        .buffer
        .as_mut_ptr()
        .offset(index as isize * (*ring).element_stride as isize)
        .cast()
}

pub unsafe fn NetRingAdvanceIndex(ring: *const NET_RING, index: u32, distance: u32) -> u32 {
    (index + distance) & (*ring).element_index_mask
}

pub unsafe fn NetRingIncrementIndex(ring: *const NET_RING, index: u32) -> u32 {
    NetRingAdvanceIndex(ring, index, 1)
}

pub unsafe fn NetRingGetPacketAtIndex(ring: *mut NET_RING, index: u32) -> *mut NET_PACKET {
    NetRingGetElementAtIndex(ring, index).cast()
}

pub unsafe fn NetRingGetFragmentAtIndex(ring: *mut NET_RING, index: u32) -> *mut NET_FRAGMENT {
    NetRingGetElementAtIndex(ring, index).cast()
}
