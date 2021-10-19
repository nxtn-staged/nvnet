use core::ptr;

use crate::{
    adapter::Adapter,
    windows::{
        km::{
            ndis::{NdisDeregisterDeviceEx, NdisGetDeviceReservedExtension},
            wdm::DEVICE_OBJECT,
        },
        shared::ndis::types::NDIS_HANDLE,
        Result,
    },
};

pub struct Device {
    adapter: Adapter,     // do not reorder: drop order
    handle: DeviceHandle, // do not reorder: drop order
}

impl Device {
    pub unsafe fn init<'a>(
        uninit: *mut Self,
        handle: DeviceHandle,
        adapter_handle: NDIS_HANDLE,
    ) -> Result<&'a mut Self> {
        Adapter::init(ptr::addr_of_mut!((*uninit).adapter), adapter_handle)?;

        ptr::addr_of_mut!((*uninit).handle).write(handle);
        Ok(&mut *uninit)
    }

    pub unsafe fn from_adapter_context<'a>(adapter_context: NDIS_HANDLE) -> &'a Self {
        &*adapter_context.0.cast()
    }

    pub unsafe fn from_adapter_context_mut<'a>(adapter_context: NDIS_HANDLE) -> &'a mut Self {
        &mut *adapter_context.0.cast()
    }

    pub unsafe fn from_device_object<'a>(device_object: *mut DEVICE_OBJECT) -> &'a mut Self {
        &mut *NdisGetDeviceReservedExtension(device_object).cast()
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn adapter_mut(&mut self) -> &mut Adapter {
        &mut self.adapter
    }
}

pub struct DeviceHandle(NDIS_HANDLE);

impl DeviceHandle {
    pub fn from_raw(device_handle: NDIS_HANDLE) -> Self {
        Self(device_handle)
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        unsafe { NdisDeregisterDeviceEx(self.0) }
    }
}
