use core::{
    default::default,
    mem::{self, MaybeUninit},
    ptr,
};

use libnveth_macros::*;

use crate::{
    adapter::{self, VEthAdapter, VEthCipherFrame},
    ioctl::*,
    windows::prelude as win,
};

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_device_prepare_hardware(
    device: win::WDFDEVICE,
    _resources_raw: win::WDFCMRESLIST,
    _resources_translated: win::WDFCMRESLIST,
) -> win::NTSTATUS {
    trace_entry!("evt_device_prepare_hardware");

    let status = (|| {
        let adapter = match VEthAdapter::init(device) {
            Err(status) => {
                return status;
            }
            Ok(adapter) => adapter,
        };
        let adapter_handle = adapter.adapter_handle;
        let link_layer_capabilities =
            win::NET_ADAPTER_LINK_LAYER_CAPABILITIES_INIT(crate::LINK_SPEED, crate::LINK_SPEED);
        unsafe {
            win::NetAdapterSetLinkLayerCapabilities(adapter_handle, &link_layer_capabilities)
        };
        unsafe { win::NetAdapterSetLinkLayerMtuSize(adapter_handle, crate::PLAIN_FRAME_DATA_SIZE) };
        let packet_filter = win::NET_ADAPTER_PACKET_FILTER_CAPABILITIES_INIT(
            win::NET_PACKET_FILTER_FLAGS::NetPacketFilterFlagDirected
                | win::NET_PACKET_FILTER_FLAGS::NetPacketFilterFlagMulticast
                | win::NET_PACKET_FILTER_FLAGS::NetPacketFilterFlagAllMulticast
                | win::NET_PACKET_FILTER_FLAGS::NetPacketFilterFlagBroadcast
                | win::NET_PACKET_FILTER_FLAGS::NetPacketFilterFlagPromiscuous,
            Some(adapter::evt_set_packet_filter),
        );
        unsafe { win::NetAdapterSetPacketFilterCapabilities(adapter_handle, &packet_filter) };
        let mut link_layer_address = win::NET_ADAPTER_LINK_LAYER_ADDRESS {
            length: 6,
            ..default()
        };
        link_layer_address.address[0..6].copy_from_slice(adapter.local_mac_addr.bytes());
        unsafe { win::NetAdapterSetPermanentLinkLayerAddress(adapter_handle, &link_layer_address) };
        unsafe { win::NetAdapterSetCurrentLinkLayerAddress(adapter_handle, &link_layer_address) };
        let tx_capabilities = win::NET_ADAPTER_TX_CAPABILITIES_INIT(1);
        let rx_capabilities = win::NET_ADAPTER_RX_CAPABILITIES_INIT_SYSTEM_MANAGED(
            mem::size_of::<VEthCipherFrame>(),
            1,
        );
        unsafe {
            win::NetAdapterSetDataPathCapabilities(
                adapter_handle,
                &tx_capabilities,
                &rx_capabilities,
            )
        };
        adapter.set_connect_state(false);
        let status = unsafe { win::NetAdapterStart(adapter_handle) };
        if !win::NT_SUCCESS(status) {
            trace_exit_status!("NetAdapterStart", status);
            return status;
        }
        win::STATUS_SUCCESS
    })();

    trace_exit_status!("evt_device_prepare_hardware", status);
    status
}

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_device_release_hardware(
    device: win::WDFDEVICE,
    _resources_translated: win::WDFCMRESLIST,
) -> win::NTSTATUS {
    trace_entry!("evt_device_release_hardware");

    let adapter = VEthAdapter::from_device_mut(device);
    adapter.drop();

    win::STATUS_SUCCESS
}

pub struct VEthFile {
    disconnect_on_close: bool,
}

impl VEthFile {
    fn from_file_mut<'a>(file_object: win::WDFFILEOBJECT) -> &'a mut Self {
        unsafe { core::intrinsics::assert_zero_valid::<Self>() };
        unsafe { &mut *veth_get_file_context(file_object.into()) }
    }
}

// WDF_DECLARE_CONTEXT_TYPE_WITH_NAME
static mut WDF_VETH_FILE_TYPE_INFO: win::WDF_OBJECT_CONTEXT_TYPE_INFO =
    win::WDF_OBJECT_CONTEXT_TYPE_INFO {
        size: mem::size_of::<win::WDF_OBJECT_CONTEXT_TYPE_INFO>() as _,
        context_name: "VEthFile\0".as_ptr(),
        context_size: mem::size_of::<VEthFile>(),
        unique_type: unsafe { &WDF_VETH_FILE_TYPE_INFO },
        evt_driver_get_unique_context_type: None,
    };

fn veth_get_file_context(handle: win::WDFOBJECT) -> *mut VEthFile {
    unsafe {
        win::WdfObjectGetTypedContextWorker(handle, WDF_VETH_FILE_TYPE_INFO.unique_type).cast()
    }
}

pub fn veth_get_file_context_type() -> *const win::WDF_OBJECT_CONTEXT_TYPE_INFO {
    unsafe { &WDF_VETH_FILE_TYPE_INFO }
}

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_file_close(file_object: win::WDFFILEOBJECT) {
    trace_entry!("evt_file_close");

    let file = VEthFile::from_file_mut(file_object);
    if file.disconnect_on_close {
        let device = unsafe { win::WdfFileObjectGetDevice(file_object) };
        let adapter = VEthAdapter::from_device_mut(device);
        adapter.set_connect_state(false);
    }
}

fn wdf_request_retrieve_input_buffer<'a, T>(
    request: win::WDFREQUEST,
) -> Result<&'a mut T, win::NTSTATUS> {
    let mut buffer = MaybeUninit::uninit();
    let status = unsafe {
        win::WdfRequestRetrieveInputBuffer(
            request,
            mem::size_of::<T>(),
            buffer.as_mut_ptr(),
            ptr::null_mut(),
        )
    };
    if !win::NT_SUCCESS(status) {
        return Err(status);
    }
    let buffer = unsafe { &mut *buffer.assume_init().cast::<T>() };
    Ok(buffer)
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub extern "system" fn evt_wdf_io_queue_io_device_control(
    queue: win::WDFQUEUE,
    request: win::WDFREQUEST,
    _output_buffer_length: usize,
    _input_buffer_length: usize,
    io_control_code: u32,
) {
    trace_entry!("evt_wdf_io_queue_io_device_control");

    let device = unsafe { win::WdfIoQueueGetDevice(queue) };
    let adapter = VEthAdapter::from_device_mut(device);

    let status = match io_control_code {
        IOCTL_VETH_SET_CONNECT_STATE => match wdf_request_retrieve_input_buffer::<bool>(request) {
            Err(status) => status,
            Ok(connected) => {
                adapter.set_connect_state(*connected);
                win::STATUS_SUCCESS
            }
        },
        IOCTL_VETH_SET_DISCONNECT_ON_CLOSE => {
            match wdf_request_retrieve_input_buffer::<bool>(request) {
                Err(status) => status,
                Ok(disconnect_on_close) => {
                    let file_object = unsafe { win::WdfRequestGetFileObject(request) };
                    let file = VEthFile::from_file_mut(file_object);
                    file.disconnect_on_close = *disconnect_on_close;
                    win::STATUS_SUCCESS
                }
            }
        }
        IOCTL_VETH_SET_LOCAL_ADDR => {
            match wdf_request_retrieve_input_buffer::<win::SOCKADDR_IN6>(request) {
                Err(status) => status,
                Ok(local_addr) => {
                    if let Err(status) = adapter.set_local_addr(local_addr.clone()) {
                        status
                    } else {
                        win::STATUS_SUCCESS
                    }
                }
            }
        }
        IOCTL_VETH_ADD_REMOTE_PEER => {
            match wdf_request_retrieve_input_buffer::<win::SOCKADDR_IN6>(request) {
                Err(status) => status,
                Ok(remote_addr) => {
                    if let Err(status) = adapter.add_peer(remote_addr.clone()) {
                        status
                    } else {
                        win::STATUS_SUCCESS
                    }
                }
            }
        }
        _ => win::STATUS_NOT_SUPPORTED,
    };

    unsafe { win::WdfRequestComplete(request, status) };
    trace_exit_status!("evt_wdf_io_queue_io_device_control", status);
}
