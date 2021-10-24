use core::{default::default, ptr::NonNull};

use crate::{
    debug::ResultExt,
    windows::{
        km::{
            ndis::{
                nbl::NET_BUFFER_LIST,
                nblapi::{
                    NdisAllocateNetBufferAndNetBufferList, NdisAllocateNetBufferListPool,
                    NdisFreeNetBufferList, NdisFreeNetBufferListPool,
                    NDIS_SIZEOF_NET_BUFFER_LIST_POOL_PARAMETERS_REVISION_1,
                    NET_BUFFER_LIST_POOL_PARAMETERS, NET_BUFFER_LIST_POOL_PARAMETERS_REVISION_1,
                },
            },
            wdm::MDL,
        },
        shared::{
            ndis::{objectheader::NDIS_OBJECT_HEADER, types::NDIS_HANDLE},
            ntddndis::{NDIS_OBJECT_TYPE_DEFAULT, NDIS_PROTOCOL_ID_DEFAULT},
            ntstatus::STATUS_INSUFFICIENT_RESOURCES,
        },
        Result,
    },
};

pub struct NblPool(NDIS_HANDLE);

impl NblPool {
    pub fn new(adapter_handle: NDIS_HANDLE, tag: u32) -> Result<Self> {
        let mut pool_params = NET_BUFFER_LIST_POOL_PARAMETERS {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_DEFAULT,
                Revision: NET_BUFFER_LIST_POOL_PARAMETERS_REVISION_1,
                Size: NDIS_SIZEOF_NET_BUFFER_LIST_POOL_PARAMETERS_REVISION_1,
            },
            ProtocolId: NDIS_PROTOCOL_ID_DEFAULT,
            fAllocateNetBuffer: true,
            PoolTag: tag,
            ..default()
        };
        let pool_handle =
            unsafe { NdisAllocateNetBufferListPool(adapter_handle, &mut pool_params) };
        NonNull::new(pool_handle.0)
            .ok_or(STATUS_INSUFFICIENT_RESOURCES)
            .context_exit("NdisAllocateNetBufferListPool")?;
        Ok(Self(pool_handle))
    }

    pub unsafe fn alloc(
        &self,
        mdl: *mut MDL,
        offset: u32,
        length: usize,
    ) -> Result<*mut NET_BUFFER_LIST> {
        let nbl = NdisAllocateNetBufferAndNetBufferList(self.0, 0, 0, mdl, offset, length);
        NonNull::new(nbl)
            .ok_or(STATUS_INSUFFICIENT_RESOURCES)
            .context_exit("NdisAllocateNetBufferAndNetBufferList")?;
        Ok(nbl)
    }

    pub unsafe fn dealloc(&self, nbl: *mut NET_BUFFER_LIST) {
        NdisFreeNetBufferList(nbl);
    }
}

impl Drop for NblPool {
    fn drop(&mut self) {
        unsafe { NdisFreeNetBufferListPool(self.0) };
    }
}
