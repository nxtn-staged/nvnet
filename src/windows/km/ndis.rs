use crate::windows::{km::wdm::MDL, shared::ntdef::PVOID};

pub type NDIS_HANDLE = PVOID;

c_type!(
    pub struct NET_BUFFER {
        pub next: *mut NET_BUFFER,   // NET_BUFFER_NEXT_NB
        pub current_mdl: *mut MDL,   // NET_BUFFER_CURRENT_MDL
        pub current_mdl_offset: u32, // NET_BUFFER_CURRENT_MDL_OFFSET
        data_length: u32,            // ...
        mdl_chain: *mut MDL,
        data_offset: u32,
        // ...
        checksum_bias: u16,
        reserved: u16,
        ndis_pool_handle: NDIS_HANDLE,
        _padding1: PVOID,
        ndis_reserved: [PVOID; 2],
        protocol_reserved: [PVOID; 6],
        miniport_reserved: [PVOID; 4],
        // ...
    }
);

c_type!(
    pub struct NET_BUFFER_LIST_CONTEXT;
);

c_type!(
    pub struct NET_BUFFER_LIST {
        pub next: *mut NET_BUFFER_LIST,        // NET_BUFFER_LIST_NEXT_NBL
        pub first_net_buffer: *mut NET_BUFFER, // NET_BUFFER_LIST_FIRST_NB
        // ...
        context: *mut NET_BUFFER_LIST_CONTEXT,
        parent_net_buffer_list: *mut NET_BUFFER_LIST,
        ndis_pool_handle: NDIS_HANDLE,
        _padding1: PVOID,
        ndis_reserved: [PVOID; 2],
        protocol_reserved: [PVOID; 4],
        pub miniport_reserved: [PVOID; 2], // NET_BUFFER_LIST_MINIPORT_RESERVED
                                           // ...
    }
);
