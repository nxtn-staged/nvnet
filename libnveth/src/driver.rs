use core::{
    mem::{self, MaybeUninit},
    ptr,
};

use libnveth_macros::*;

use crate::{
    adapter::{self, VEthAdapter, VEthAdapterPtr},
    device,
    socket::UdpSocket,
    windows::prelude as win,
};

static mut DEVICE_INDEX: u16 = 0;

fn unicode_string_init(chars: &mut [u16]) -> win::UNICODE_STRING {
    let size = mem::size_of_val(chars) as _;
    win::UNICODE_STRING {
        length: size,
        maximum_length: size,
        buffer: chars.as_mut_ptr(),
    }
}

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_driver_device_add(
    _driver: win::WDFDRIVER,
    mut device_init: *mut win::WDFDEVICE_INIT,
) -> win::NTSTATUS {
    trace_entry!("evt_driver_device_add");

    let status = (|| {
        let status = unsafe { win::NetDeviceInitConfig(device_init) };
        if !win::NT_SUCCESS(status) {
            trace_exit_status!("NetDeviceInitConfig", status);
            return status;
        }
        let mut pnp_power_callbacks = win::WDF_PNPPOWER_EVENT_CALLBACKS_INIT();
        pnp_power_callbacks.evt_device_prepare_hardware = Some(device::evt_device_prepare_hardware);
        pnp_power_callbacks.evt_device_release_hardware = Some(device::evt_device_release_hardware);
        unsafe { win::WdfDeviceInitSetPnpPowerEventCallbacks(device_init, &pnp_power_callbacks) };
        let file_config = win::WDF_FILEOBJECT_CONFIG_INIT(None, Some(device::evt_file_close), None);
        let mut file_attributes = win::WDF_OBJECT_ATTRIBUTES_INIT();
        file_attributes.context_type_info = device::veth_get_file_context_type();
        unsafe {
            win::WdfDeviceInitSetFileObjectConfig(device_init, &file_config, &file_attributes)
        };
        let mut device_attributes = win::WDF_OBJECT_ATTRIBUTES_INIT();
        device_attributes.context_type_info = adapter::veth_get_adapter_ptr_context_type();
        let mut device = MaybeUninit::uninit();
        let status = unsafe {
            win::WdfDeviceCreate(&mut device_init, &device_attributes, device.as_mut_ptr())
        };
        if !win::NT_SUCCESS(status) {
            trace_exit_status!("WdfDeviceCreate", status);
            return status;
        }
        let device = unsafe { device.assume_init() };
        let adapter_init = unsafe { win::NetAdapterInitAllocate(device) };
        if adapter_init.is_null() {
            let status = win::STATUS_INSUFFICIENT_RESOURCES;
            trace_exit_status!("NetAdapterInitAllocate", status);
            return status;
        }
        let mut adapter_handle = MaybeUninit::uninit();
        let status = (|| {
            let datapath_callbacks = win::NET_ADAPTER_DATAPATH_CALLBACKS_INIT(
                Some(adapter::evt_adapter_create_tx_queue),
                Some(adapter::evt_adapter_create_rx_queue),
            );
            unsafe { win::NetAdapterInitSetDatapathCallbacks(adapter_init, &datapath_callbacks) };
            let mut adapter_attributes = win::WDF_OBJECT_ATTRIBUTES_INIT();
            adapter_attributes.context_type_info = adapter::veth_get_adapter_context_type();
            let status = unsafe {
                win::NetAdapterCreate(
                    adapter_init,
                    &adapter_attributes,
                    adapter_handle.as_mut_ptr(),
                )
            };
            if !win::NT_SUCCESS(status) {
                trace_exit_status!("NetAdapterCreate", status);
                return status;
            }
            win::STATUS_SUCCESS
        })();
        unsafe { win::NetAdapterInitFree(adapter_init) };
        if !win::NT_SUCCESS(status) {
            return status;
        }
        let adapter_handle = unsafe { adapter_handle.assume_init() };
        let adapter = VEthAdapter::pre_init(adapter_handle);
        VEthAdapterPtr::init(device, adapter);
        let mut queue_config = win::WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(
            win::WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchSequential,
        );
        queue_config.evt_io_device_control = Some(device::evt_wdf_io_queue_io_device_control);
        let status =
            unsafe { win::WdfIoQueueCreate(device, &queue_config, ptr::null(), ptr::null_mut()) };
        if !win::NT_SUCCESS(status) {
            trace_exit_status!("WdfIoQueueCreate", status);
            return status;
        }
        let mut symbolic_link_name = utf16!(br"\DosDevices\Global\NVEth0");
        *symbolic_link_name.last_mut().unwrap() += unsafe { DEVICE_INDEX };
        let symbolic_link_name = unicode_string_init(&mut symbolic_link_name);
        let status = unsafe { win::WdfDeviceCreateSymbolicLink(device, &symbolic_link_name) };
        if !win::NT_SUCCESS(status) {
            trace_exit_status!("WdfDeviceCreateSymbolicLink", status);
            return status;
        }
        unsafe { DEVICE_INDEX += 1 };
        win::STATUS_SUCCESS
    })();

    trace_exit_status!("evt_driver_device_add", status);
    status
}

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_driver_device_unload(_driver: win::WDFDRIVER) {
    trace_entry!("evt_driver_device_unload");

    UdpSocket::deregister();
}
