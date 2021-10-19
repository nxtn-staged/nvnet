use crate::{
    windows::{
        km::{ndis::nbl::NET_BUFFER_LIST, wdm::MDL},
        shared::ndis::{objectheader::NDIS_OBJECT_HEADER, types::NDIS_HANDLE},
    },
    RTL_SIZEOF_THROUGH_FIELD,
};

// L20
pub const NET_BUFFER_LIST_POOL_PARAMETERS_REVISION_1: u8 = 1;

c_type!(
    pub struct NET_BUFFER_LIST_POOL_PARAMETERS {
        pub Header: NDIS_OBJECT_HEADER,
        pub ProtocolId: u8,
        pub fAllocateNetBuffer: bool,
        pub ContextSize: u16,
        pub PoolTag: u32,
        pub DataSize: u32,
    }
);

pub const NDIS_SIZEOF_NET_BUFFER_LIST_POOL_PARAMETERS_REVISION_1: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NET_BUFFER_LIST_POOL_PARAMETERS, DataSize);

extern "system" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn NdisAllocateNetBufferListPool(
        NdisHandle: NDIS_HANDLE,
        Parameters: *const NET_BUFFER_LIST_POOL_PARAMETERS,
    ) -> NDIS_HANDLE;

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn NdisFreeNetBufferListPool(PoolHandle: NDIS_HANDLE);

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn NdisAllocateNetBufferAndNetBufferList(
        PoolHandle: NDIS_HANDLE,
        ContextSize: u16,
        ContextBackFill: u16,
        MdlChain: *mut MDL,
        DataOffset: u32,
        DataLength: usize,
    ) -> *mut NET_BUFFER_LIST;

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn NdisFreeNetBufferList(NetBufferList: *mut NET_BUFFER_LIST);
}
