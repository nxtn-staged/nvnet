use core::{ffi::c_void, ptr};

use crate::{
    os::socket::SocketClient,
    windows::{
        km::{ndis::NdisMDeregisterMiniportDriver, wsk::WSK_DATAGRAM_INDICATION},
        shared::{ndis::types::NDIS_HANDLE, ntdef::NTSTATUS},
        Result,
    },
};

pub struct Driver {
    handle: DriverHandle,
    socket_client: SocketClient,
}

impl Driver {
    pub unsafe fn init<'a>(
        uninit: *mut Self,
        handle: DriverHandle,
        evt_recv_from: extern "system" fn(
            *mut c_void,
            u32,
            *mut WSK_DATAGRAM_INDICATION,
        ) -> NTSTATUS,
    ) -> Result<&'a mut Self> {
        SocketClient::init(ptr::addr_of_mut!((*uninit).socket_client), evt_recv_from)?;
        ptr::addr_of_mut!((*uninit).handle).write(handle);
        Ok(&mut *uninit)
    }

    pub fn handle(&self) -> &DriverHandle {
        &self.handle
    }

    pub fn socket_client(&self) -> &SocketClient {
        &self.socket_client
    }
}

pub struct DriverHandle(NDIS_HANDLE);

impl DriverHandle {
    pub fn from_raw(driver_handle: NDIS_HANDLE) -> Self {
        Self(driver_handle)
    }

    pub fn as_raw(&self) -> NDIS_HANDLE {
        self.0
    }
}

impl Drop for DriverHandle {
    fn drop(&mut self) {
        unsafe { NdisMDeregisterMiniportDriver(self.0) };
    }
}
