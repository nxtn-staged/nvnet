use core::ffi::c_void;

use crate::windows::{km::wdm::MDL, shared::ndis::types::NDIS_HANDLE};

// L91
c_type!(
    pub struct NET_BUFFER {
        pub next: *mut Self,
        pub current_mdl: *mut MDL,
        pub current_mdl_offset: u32,
        _padding0: u32,
        pub data_length: u32,
        pub mdl_chain: *mut MDL,
        pub data_offset: u32,
        pub checksum_bias: u16,
        pub reserved: u16,
        pub ndis_pool_handle: NDIS_HANDLE,
        _padding1: *mut c_void,
        pub ndis_reserved: [*mut c_void; 2],
        pub protocol_reserved: [*mut c_void; 6],
        pub miniport_reserved: [*mut c_void; 4],
        // ...
    }
);

// L141
c_type!(
    pub struct NET_BUFFER_LIST_CONTEXT;
);

// L171
c_type!(
    pub struct NET_BUFFER_LIST {
        pub next: *mut Self,
        pub first_net_buffer: *mut NET_BUFFER,
        pub context: *mut NET_BUFFER_LIST_CONTEXT,
        pub parent_net_buffer_list: *mut NET_BUFFER_LIST,
        pub ndis_pool_handle: NDIS_HANDLE,
        _padding0: *mut c_void,
        pub ndis_reserved: [*mut c_void; 2],
        pub protocol_reserved: [*mut c_void; 4],
        pub miniport_reserved: [*mut c_void; 2],
        // TODO
        pub Scratch: *mut c_void,
        pub SourceHandle: NDIS_HANDLE,
        pub NblFlags: u32,
        // ...
    }
);
