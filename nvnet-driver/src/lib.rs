#![no_std]
#![feature(arbitrary_self_types)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(const_size_of_val_raw)]
#![feature(default_free_fn)]
#![feature(layout_for_ptr)]
#![feature(maybe_uninit_extra)]
#![feature(option_result_unwrap_unchecked)]
#![feature(untagged_unions)]

#[macro_use]
mod debug;

mod adapter;
mod device;
mod driver;
mod linked;
mod ndis;
mod os;
#[cfg(not(test))]
mod panic;
mod windows;

use core::{
    default::default,
    ffi::c_void,
    mem::{self, MaybeUninit},
    ops::AddAssign,
    ptr,
    sync::atomic::Ordering::Relaxed,
};

use crate::{
    adapter::Adapter,
    debug::ResultExt,
    device::{Device, DeviceHandle},
    driver::{Driver, DriverHandle},
    linked::{LinkedIter, LinkedQueue},
    ndis::NblChain,
    os::request::Request,
    windows::{
        km::{
            ndis::{
                nbl::{NET_BUFFER, NET_BUFFER_LIST},
                nblreceive::{NDIS_RECEIVE_FLAGS_DISPATCH_LEVEL, NDIS_RETURN_FLAGS_DISPATCH_LEVEL},
                nblsend::NDIS_SEND_FLAGS_DISPATCH_LEVEL,
                oidrequest::{NDIS_OID_REQUEST, NDIS_OID_REQUEST_0_0, NDIS_OID_REQUEST_0_1},
                NdisGetDeviceReservedExtension, NdisMIndicateReceiveNetBufferLists,
                NdisMRegisterMiniportDriver, NdisMSetMiniportAttributes, NdisRegisterDeviceEx,
                NDIS_DEVICE_OBJECT_ATTRIBUTES, NDIS_DEVICE_OBJECT_ATTRIBUTES_REVISION_1,
                NDIS_HALT_ACTION, NDIS_INTERFACE_TYPE, NDIS_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES,
                NDIS_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES_REVISION_2,
                NDIS_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES,
                NDIS_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES_REVISION_2,
                NDIS_MINIPORT_DRIVER_CHARACTERISTICS,
                NDIS_MINIPORT_DRIVER_CHARACTERISTICS_REVISION_2, NDIS_MINIPORT_INIT_PARAMETERS,
                NDIS_MINIPORT_MAJOR_VERSION, NDIS_MINIPORT_MINOR_VERSION,
                NDIS_MINIPORT_PAUSE_PARAMETERS, NDIS_MINIPORT_RESTART_PARAMETERS,
                NDIS_SHUTDOWN_ACTION, NDIS_SIZEOF_DEVICE_OBJECT_ATTRIBUTES_REVISION_1,
                NDIS_SIZEOF_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES_REVISION_2,
                NDIS_SIZEOF_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES_REVISION_2,
                NDIS_SIZEOF_MINIPORT_DRIVER_CHARACTERISTICS_REVISION_2,
                NDIS_STATISTICS_BROADCAST_BYTES_RCV_SUPPORTED,
                NDIS_STATISTICS_BROADCAST_BYTES_XMIT_SUPPORTED,
                NDIS_STATISTICS_BROADCAST_FRAMES_RCV_SUPPORTED,
                NDIS_STATISTICS_BROADCAST_FRAMES_XMIT_SUPPORTED,
                NDIS_STATISTICS_BYTES_RCV_SUPPORTED, NDIS_STATISTICS_BYTES_XMIT_SUPPORTED,
                NDIS_STATISTICS_DIRECTED_BYTES_RCV_SUPPORTED,
                NDIS_STATISTICS_DIRECTED_BYTES_XMIT_SUPPORTED,
                NDIS_STATISTICS_DIRECTED_FRAMES_RCV_SUPPORTED,
                NDIS_STATISTICS_DIRECTED_FRAMES_XMIT_SUPPORTED,
                NDIS_STATISTICS_MULTICAST_BYTES_RCV_SUPPORTED,
                NDIS_STATISTICS_MULTICAST_BYTES_XMIT_SUPPORTED,
                NDIS_STATISTICS_MULTICAST_FRAMES_RCV_SUPPORTED,
                NDIS_STATISTICS_MULTICAST_FRAMES_XMIT_SUPPORTED,
                NDIS_STATISTICS_RCV_DISCARDS_SUPPORTED, NDIS_STATISTICS_RCV_ERROR_SUPPORTED,
                NDIS_STATISTICS_XMIT_DISCARDS_SUPPORTED, NDIS_STATISTICS_XMIT_ERROR_SUPPORTED,
                NET_DEVICE_PNP_EVENT,
            },
            ntifs::RtlRandomEx,
            wdm::{
                IoGetCurrentIrpStackLocation, DEVICE_OBJECT, DRIVER_OBJECT, IRP, IRP_MJ_CLOSE,
                IRP_MJ_CREATE, IRP_MJ_DEVICE_CONTROL, IRP_MJ_MAXIMUM_FUNCTION,
                PDRIVER_DISPATCH_PAGED,
            },
            wsk::{WSK_BUF, WSK_BUF_LIST, WSK_DATAGRAM_INDICATION, WSK_FLAG_AT_DISPATCH_LEVEL},
        },
        shared::{
            ifdef::{NET_IF_ACCESS_TYPE, NET_IF_CONNECTION_TYPE, NET_IF_DIRECTION_TYPE},
            ipifcons::IF_TYPE_ETHERNET_CSMACD,
            ndis::{
                ndisport::{NDIS_DEFAULT_PORT_NUMBER, NDIS_PORT_NUMBER},
                objectheader::NDIS_OBJECT_HEADER,
                oidtypes::NDIS_REQUEST_TYPE,
                status::{
                    NDIS_STATUS_INVALID_DATA, NDIS_STATUS_INVALID_LENGTH,
                    NDIS_STATUS_NOT_SUPPORTED, NDIS_STATUS_SUCCESS,
                },
                types::{NDIS_HANDLE, NDIS_STATUS},
            },
            ntddndis::{
                NDIS_INTERRUPT_MODERATION, NDIS_INTERRUPT_MODERATION_PARAMETERS,
                NDIS_INTERRUPT_MODERATION_PARAMETERS_REVISION_1, NDIS_MAC_OPTION_NO_LOOPBACK,
                NDIS_MAX_PHYS_ADDRESS_LENGTH, NDIS_MEDIA_CONNECT_STATE, NDIS_MEDIA_DUPLEX_STATE,
                NDIS_MEDIUM, NDIS_OBJECT_TYPE_DEFAULT, NDIS_OBJECT_TYPE_DEVICE_OBJECT_ATTRIBUTES,
                NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES,
                NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES,
                NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS, NDIS_PACKET_TYPE_ALL_MULTICAST,
                NDIS_PACKET_TYPE_BROADCAST, NDIS_PACKET_TYPE_DIRECTED, NDIS_PACKET_TYPE_MULTICAST,
                NDIS_PACKET_TYPE_PROMISCUOUS, NDIS_PHYSICAL_MEDIUM, NDIS_PM_CAPABILITIES,
                NDIS_PM_CAPABILITIES_REVISION_2,
                NDIS_SIZEOF_INTERRUPT_MODERATION_PARAMETERS_REVISION_1,
                NDIS_SIZEOF_NDIS_PM_CAPABILITIES_REVISION_2,
                NDIS_SIZEOF_STATISTICS_INFO_REVISION_1,
                NDIS_STATISTICS_FLAGS_VALID_BROADCAST_BYTES_RCV,
                NDIS_STATISTICS_FLAGS_VALID_BROADCAST_BYTES_XMIT,
                NDIS_STATISTICS_FLAGS_VALID_BROADCAST_FRAMES_RCV,
                NDIS_STATISTICS_FLAGS_VALID_BROADCAST_FRAMES_XMIT,
                NDIS_STATISTICS_FLAGS_VALID_BYTES_RCV, NDIS_STATISTICS_FLAGS_VALID_BYTES_XMIT,
                NDIS_STATISTICS_FLAGS_VALID_DIRECTED_BYTES_RCV,
                NDIS_STATISTICS_FLAGS_VALID_DIRECTED_BYTES_XMIT,
                NDIS_STATISTICS_FLAGS_VALID_DIRECTED_FRAMES_RCV,
                NDIS_STATISTICS_FLAGS_VALID_DIRECTED_FRAMES_XMIT,
                NDIS_STATISTICS_FLAGS_VALID_MULTICAST_BYTES_RCV,
                NDIS_STATISTICS_FLAGS_VALID_MULTICAST_BYTES_XMIT,
                NDIS_STATISTICS_FLAGS_VALID_MULTICAST_FRAMES_RCV,
                NDIS_STATISTICS_FLAGS_VALID_MULTICAST_FRAMES_XMIT,
                NDIS_STATISTICS_FLAGS_VALID_RCV_DISCARDS, NDIS_STATISTICS_FLAGS_VALID_RCV_ERROR,
                NDIS_STATISTICS_FLAGS_VALID_XMIT_DISCARDS, NDIS_STATISTICS_FLAGS_VALID_XMIT_ERROR,
                NDIS_STATISTICS_INFO, NDIS_STATISTICS_INFO_REVISION_1,
                NDIS_SUPPORTED_PAUSE_FUNCTIONS, OID_GEN_CURRENT_PACKET_FILTER,
                OID_GEN_INTERRUPT_MODERATION, OID_GEN_MAXIMUM_TOTAL_SIZE, OID_GEN_RCV_OK,
                OID_GEN_STATISTICS, OID_GEN_XMIT_OK,
            },
            ntdef::{NTSTATUS, UNICODE_STRING},
            ntstatus::{STATUS_NOT_SUPPORTED, STATUS_PENDING, STATUS_SUCCESS},
            ws2ipdef::SOCKADDR_IN6,
        },
        OkExt, Result, UnicodeStringExt,
    },
};

macro_rules! utf16 {
    ($bytes:expr) => {{
        const BYTES: &[u8] = $bytes;
        const LEN: usize = BYTES.len();
        let mut chars = [0u16; LEN];
        let mut i = 0;
        while i < LEN {
            chars[i] = BYTES[i] as u16;
            i += 1;
        }

        chars
    }};
}

const LINK_SPEED: u64 = 10_000_000_000; // 10.0 Gbps;

static mut DRIVER: MaybeUninit<Driver> = MaybeUninit::uninit();

static mut DEVICE_INDEX: u16 = 0;

// #[irql_requires_same]
// #[irql_requires(PASSIVE_LEVEL)]
#[no_mangle]
extern "system" fn DriverEntry(
    driver_object: *mut DRIVER_OBJECT,
    registry_path: *mut UNICODE_STRING,
) -> NTSTATUS {
    (|| {
        let mut driver_chars = NDIS_MINIPORT_DRIVER_CHARACTERISTICS {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS,
                Revision: NDIS_MINIPORT_DRIVER_CHARACTERISTICS_REVISION_2,
                Size: NDIS_SIZEOF_MINIPORT_DRIVER_CHARACTERISTICS_REVISION_2,
            },
            MajorNdisVersion: NDIS_MINIPORT_MAJOR_VERSION,
            MinorNdisVersion: NDIS_MINIPORT_MINOR_VERSION,
            MajorDriverVersion: 0,
            MinorDriverVersion: 0,
            InitializeHandlerEx: Some(evt_initialize),
            HaltHandlerEx: Some(evt_halt),
            UnloadHandler: Some(evt_unload),
            PauseHandler: Some(evt_pause),
            RestartHandler: Some(evt_restart),
            OidRequestHandler: Some(evt_oid_request),
            SendNetBufferListsHandler: Some(evt_send_net_buffer_lists),
            ReturnNetBufferListsHandler: Some(evt_return_net_buffer_lists),
            CancelSendHandler: Some(evt_cancel_send),
            DevicePnPEventNotifyHandler: Some(evt_device_pnp_event_notify),
            ShutdownHandlerEx: Some(evt_shutdown),
            CancelOidRequestHandler: Some(evt_cancel_oid_request),
            ..default()
        };
        let mut driver_handle = MaybeUninit::uninit();
        unsafe {
            NdisMRegisterMiniportDriver(
                driver_object,
                registry_path,
                default(),
                &mut driver_chars,
                driver_handle.as_mut_ptr(),
            )
            .ok()
            .context_exit("NdisMRegisterMiniportDriver")?
        };
        let driver_handle = unsafe { driver_handle.assume_init() };
        let driver_handle = DriverHandle::from_raw(driver_handle);
        unsafe { Driver::init(DRIVER.as_mut_ptr(), driver_handle, evt_recv_from)? };
        Ok(())
    })()
    .into()
}

extern "system" fn evt_unload(_driver_object: *mut DRIVER_OBJECT) {
    unsafe { DRIVER.assume_init_drop() };
}

trait NblExt {
    unsafe fn wsk_datagram_mut(&mut self) -> *mut *mut WSK_DATAGRAM_INDICATION;
}

impl NblExt for NET_BUFFER_LIST {
    unsafe fn wsk_datagram_mut(&mut self) -> *mut *mut WSK_DATAGRAM_INDICATION {
        ptr::addr_of_mut!(self.miniport_reserved[0]).cast()
    }
}

trait NbExt {
    unsafe fn wsk_buf_list_mut(&mut self) -> *mut WSK_BUF_LIST;
}

impl NbExt for NET_BUFFER {
    unsafe fn wsk_buf_list_mut(&mut self) -> *mut WSK_BUF_LIST {
        ptr::addr_of_mut!(self.miniport_reserved[0]).cast()
    }
}

extern "system" fn evt_initialize(
    adapter_handle: NDIS_HANDLE,
    _driver_context: NDIS_HANDLE,
    _init_params: *mut NDIS_MINIPORT_INIT_PARAMETERS,
) -> NDIS_STATUS {
    (|| {
        let device_index = unsafe {
            let device_index = DEVICE_INDEX;
            DEVICE_INDEX += 1;
            device_index
        };
        let mut device_name = unsafe {
            let mut device_name = utf16!(br"\Device\NVNet0");
            device_name
                .last_mut()
                .unwrap_unchecked()
                .add_assign(device_index);
            device_name
        };
        let mut device_name = unsafe { UNICODE_STRING::new(&mut device_name) };
        let mut symbolic_name = unsafe {
            let mut symbolic_name = utf16!(br"\DosDevices\Global\NVNet0");
            symbolic_name
                .last_mut()
                .unwrap_unchecked()
                .add_assign(device_index);
            symbolic_name
        };
        let mut symbolic_name = unsafe { UNICODE_STRING::new(&mut symbolic_name) };
        let mut major_functions = {
            let mut major_functions = [PDRIVER_DISPATCH_PAGED::None; IRP_MJ_MAXIMUM_FUNCTION];
            major_functions[IRP_MJ_CREATE] = Some(evt_file_create);
            major_functions[IRP_MJ_CLOSE] = Some(evt_file_close);
            major_functions[IRP_MJ_DEVICE_CONTROL] = Some(evt_io_device_control);
            major_functions
        };
        let mut device_attrs = NDIS_DEVICE_OBJECT_ATTRIBUTES {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_DEVICE_OBJECT_ATTRIBUTES,
                Revision: NDIS_DEVICE_OBJECT_ATTRIBUTES_REVISION_1,
                Size: NDIS_SIZEOF_DEVICE_OBJECT_ATTRIBUTES_REVISION_1,
            },
            DeviceName: &mut device_name,
            SymbolicName: &mut symbolic_name,
            MajorFunctions: major_functions.as_mut_ptr(),
            ExtensionSize: mem::size_of::<Device>() as u32,
            ..default()
        };
        let mut device_object = MaybeUninit::uninit();
        let mut device_handle = MaybeUninit::uninit();
        unsafe {
            NdisRegisterDeviceEx(
                DRIVER.assume_init_ref().handle().as_raw(),
                &mut device_attrs,
                device_object.as_mut_ptr(),
                device_handle.as_mut_ptr(),
            )
            .ok()
            .context_exit("NdisRegisterDeviceEx")?
        };
        let device_object = unsafe { device_object.assume_init() };
        let device_handle = unsafe { device_handle.assume_init() };
        let device_handle = DeviceHandle::from_raw(device_handle);
        let adapter_context = unsafe { NdisGetDeviceReservedExtension(device_object) };
        let mut adapter_registration = NDIS_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES,
                Revision: NDIS_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES_REVISION_2,
                Size: NDIS_SIZEOF_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES_REVISION_2,
            },
            MiniportAdapterContext: NDIS_HANDLE(adapter_context),
            InterfaceType: NDIS_INTERFACE_TYPE::NdisInterfaceInternal,
            ..default()
        };
        unsafe {
            NdisMSetMiniportAttributes(
                adapter_handle,
                &mut adapter_registration as *mut _ as *mut _,
            )
            .ok()
            .context_exit("NdisMSetMiniportAttributes")?
        };
        let mac_addr = {
            let mut seed = 42;
            let random = unsafe { RtlRandomEx(&mut seed) };
            let random = random.to_ne_bytes();
            let mut mac_addr = [0; NDIS_MAX_PHYS_ADDRESS_LENGTH];
            mac_addr[0] = 0x02;
            mac_addr[1] = 0x00;
            mac_addr[2..6].copy_from_slice(&random);
            mac_addr
        };
        let mut oid_list = [
            OID_GEN_CURRENT_PACKET_FILTER,
            OID_GEN_MAXIMUM_TOTAL_SIZE,
            OID_GEN_INTERRUPT_MODERATION,
            OID_GEN_XMIT_OK,
            OID_GEN_RCV_OK,
            OID_GEN_STATISTICS,
        ];
        let mut pm_capabilities = NDIS_PM_CAPABILITIES {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_DEFAULT,
                Revision: NDIS_PM_CAPABILITIES_REVISION_2,
                Size: NDIS_SIZEOF_NDIS_PM_CAPABILITIES_REVISION_2,
            },
            ..default()
        };
        let mut adapter_general = NDIS_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES,
                Revision: NDIS_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES_REVISION_2,
                Size: NDIS_SIZEOF_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES_REVISION_2,
            },
            MediaType: NDIS_MEDIUM::NdisMedium802_3,
            PhysicalMediumType: NDIS_PHYSICAL_MEDIUM::NdisPhysicalMediumUnspecified,
            MtuSize: adapter::MAX_FRAME_MTU_SIZE,
            MaxXmitLinkSpeed: LINK_SPEED,
            XmitLinkSpeed: LINK_SPEED,
            MaxRcvLinkSpeed: LINK_SPEED,
            RcvLinkSpeed: LINK_SPEED,
            MediaConnectState: NDIS_MEDIA_CONNECT_STATE::MediaConnectStateDisconnected,
            MediaDuplexState: NDIS_MEDIA_DUPLEX_STATE::MediaDuplexStateFull,
            LookaheadSize: adapter::MAX_FRAME_MTU_SIZE,
            MacOptions: NDIS_MAC_OPTION_NO_LOOPBACK,
            SupportedPacketFilters: NDIS_PACKET_TYPE_DIRECTED
                | NDIS_PACKET_TYPE_MULTICAST
                | NDIS_PACKET_TYPE_ALL_MULTICAST
                | NDIS_PACKET_TYPE_BROADCAST
                | NDIS_PACKET_TYPE_PROMISCUOUS,
            MacAddressLength: 6,
            PermanentMacAddress: mac_addr,
            CurrentMacAddress: mac_addr,
            AccessType: NET_IF_ACCESS_TYPE::NET_IF_ACCESS_BROADCAST,
            DirectionType: NET_IF_DIRECTION_TYPE::NET_IF_DIRECTION_SENDRECEIVE,
            ConnectionType: NET_IF_CONNECTION_TYPE::NET_IF_CONNECTION_DEDICATED,
            IfType: IF_TYPE_ETHERNET_CSMACD,
            IfConnectorPresent: false,
            SupportedStatistics: NDIS_STATISTICS_XMIT_ERROR_SUPPORTED
                | NDIS_STATISTICS_RCV_ERROR_SUPPORTED
                | NDIS_STATISTICS_DIRECTED_BYTES_XMIT_SUPPORTED
                | NDIS_STATISTICS_DIRECTED_FRAMES_XMIT_SUPPORTED
                | NDIS_STATISTICS_MULTICAST_BYTES_XMIT_SUPPORTED
                | NDIS_STATISTICS_MULTICAST_FRAMES_XMIT_SUPPORTED
                | NDIS_STATISTICS_BROADCAST_BYTES_XMIT_SUPPORTED
                | NDIS_STATISTICS_BROADCAST_FRAMES_XMIT_SUPPORTED
                | NDIS_STATISTICS_DIRECTED_BYTES_RCV_SUPPORTED
                | NDIS_STATISTICS_DIRECTED_FRAMES_RCV_SUPPORTED
                | NDIS_STATISTICS_MULTICAST_BYTES_RCV_SUPPORTED
                | NDIS_STATISTICS_MULTICAST_FRAMES_RCV_SUPPORTED
                | NDIS_STATISTICS_BROADCAST_BYTES_RCV_SUPPORTED
                | NDIS_STATISTICS_BROADCAST_FRAMES_RCV_SUPPORTED
                | NDIS_STATISTICS_BYTES_RCV_SUPPORTED
                | NDIS_STATISTICS_BYTES_XMIT_SUPPORTED
                | NDIS_STATISTICS_RCV_DISCARDS_SUPPORTED
                | NDIS_STATISTICS_XMIT_DISCARDS_SUPPORTED,
            SupportedPauseFunctions: NDIS_SUPPORTED_PAUSE_FUNCTIONS::NdisPauseFunctionsUnsupported,
            SupportedOidList: oid_list.as_mut_ptr(),
            SupportedOidListLength: mem::size_of_val(&oid_list) as u32,
            PowerManagementCapabilitiesEx: &mut pm_capabilities,
            ..default()
        };
        unsafe {
            NdisMSetMiniportAttributes(adapter_handle, &mut adapter_general as *mut _ as *mut _)
                .ok()
                .context_exit("NdisMSetMiniportAttributes")?
        };
        unsafe { Device::init(adapter_context.cast(), device_handle, adapter_handle)? };
        Ok(())
    })()
    .into()
}

extern "system" fn evt_halt(adapter_context: NDIS_HANDLE, _halt_action: NDIS_HALT_ACTION) {
    let device = unsafe { Device::from_adapter_context_mut(adapter_context) };
    unsafe { ptr::drop_in_place(device) };
}

extern "system" fn evt_pause(
    _adapter_context: NDIS_HANDLE,
    _pause_params: *mut NDIS_MINIPORT_PAUSE_PARAMETERS,
) -> NDIS_STATUS {
    NDIS_STATUS_SUCCESS
}

extern "system" fn evt_restart(
    _adapter_context: NDIS_HANDLE,
    _restart_params: *mut NDIS_MINIPORT_RESTART_PARAMETERS,
) -> NDIS_STATUS {
    NDIS_STATUS_SUCCESS
}

extern "system" fn evt_oid_request(
    adapter_context: NDIS_HANDLE,
    oid_request: *mut NDIS_OID_REQUEST,
) -> NDIS_STATUS {
    let adapter = unsafe { Device::from_adapter_context(adapter_context).adapter() };
    (|| match unsafe { (*oid_request).RequestType } {
        NDIS_REQUEST_TYPE::NdisRequestQueryInformation
        | NDIS_REQUEST_TYPE::NdisRequestQueryStatistics => {
            unsafe fn write<T>(
                request_data: &mut NDIS_OID_REQUEST_0_0,
                f: impl FnOnce() -> T,
            ) -> Result<()> {
                let size = mem::size_of::<T>() as u32;
                if request_data.InformationBufferLength < size {
                    request_data.BytesNeeded = size;
                    return Err(NDIS_STATUS_INVALID_LENGTH.into());
                }
                request_data.InformationBuffer.cast::<T>().write(f());
                request_data.BytesWritten = size;
                Ok(())
            }

            let request_data = unsafe { &mut (*oid_request).DATA.QUERY_INFORMATION };
            match request_data.Oid {
                OID_GEN_MAXIMUM_TOTAL_SIZE => {
                    let result = || adapter::MAX_FRAME_DATA_SIZE;
                    unsafe { write(request_data, result)? };
                    Ok(())
                }
                OID_GEN_INTERRUPT_MODERATION => {
                    let result = || NDIS_INTERRUPT_MODERATION_PARAMETERS {
                        Header: NDIS_OBJECT_HEADER {
                            Type: NDIS_OBJECT_TYPE_DEFAULT,
                            Revision: NDIS_INTERRUPT_MODERATION_PARAMETERS_REVISION_1,
                            Size: NDIS_SIZEOF_INTERRUPT_MODERATION_PARAMETERS_REVISION_1,
                        },
                        InterruptModeration:
                            NDIS_INTERRUPT_MODERATION::NdisInterruptModerationNotSupported,
                        ..default()
                    };
                    unsafe { write(request_data, result)? };
                    Ok(())
                }
                OID_GEN_XMIT_OK => {
                    let result = || {
                        adapter.tx_frames_unicast.load(Relaxed)
                            + adapter.tx_frames_multicast.load(Relaxed)
                            + adapter.tx_frames_broadcast.load(Relaxed)
                    };
                    unsafe { write(request_data, result)? };
                    Ok(())
                }
                OID_GEN_RCV_OK => {
                    let result = || {
                        adapter.rx_frames_unicast.load(Relaxed)
                            + adapter.rx_frames_multicast.load(Relaxed)
                            + adapter.rx_frames_broadcast.load(Relaxed)
                    };
                    unsafe { write(request_data, result)? };
                    Ok(())
                }
                OID_GEN_STATISTICS => {
                    let result = || {
                        let tx_bytes_unicast = adapter.tx_bytes_unicast.load(Relaxed);
                        let tx_bytes_multicast = adapter.tx_bytes_multicast.load(Relaxed);
                        let tx_bytes_broadcast = adapter.tx_bytes_broadcast.load(Relaxed);
                        let rx_bytes_unicast = adapter.rx_bytes_unicast.load(Relaxed);
                        let rx_bytes_multicast = adapter.rx_bytes_multicast.load(Relaxed);
                        let rx_bytes_broadcast = adapter.rx_bytes_broadcast.load(Relaxed);
                        NDIS_STATISTICS_INFO {
                            Header: NDIS_OBJECT_HEADER {
                                Type: NDIS_OBJECT_TYPE_DEFAULT,
                                Revision: NDIS_STATISTICS_INFO_REVISION_1,
                                Size: NDIS_SIZEOF_STATISTICS_INFO_REVISION_1,
                            },
                            SupportedStatistics: NDIS_STATISTICS_FLAGS_VALID_DIRECTED_FRAMES_RCV
                                | NDIS_STATISTICS_FLAGS_VALID_MULTICAST_FRAMES_RCV
                                | NDIS_STATISTICS_FLAGS_VALID_BROADCAST_FRAMES_RCV
                                | NDIS_STATISTICS_FLAGS_VALID_BYTES_RCV
                                | NDIS_STATISTICS_FLAGS_VALID_RCV_DISCARDS
                                | NDIS_STATISTICS_FLAGS_VALID_RCV_ERROR
                                | NDIS_STATISTICS_FLAGS_VALID_DIRECTED_FRAMES_XMIT
                                | NDIS_STATISTICS_FLAGS_VALID_MULTICAST_FRAMES_XMIT
                                | NDIS_STATISTICS_FLAGS_VALID_BROADCAST_FRAMES_XMIT
                                | NDIS_STATISTICS_FLAGS_VALID_BYTES_XMIT
                                | NDIS_STATISTICS_FLAGS_VALID_XMIT_ERROR
                                | NDIS_STATISTICS_FLAGS_VALID_XMIT_DISCARDS
                                | NDIS_STATISTICS_FLAGS_VALID_DIRECTED_BYTES_RCV
                                | NDIS_STATISTICS_FLAGS_VALID_MULTICAST_BYTES_RCV
                                | NDIS_STATISTICS_FLAGS_VALID_BROADCAST_BYTES_RCV
                                | NDIS_STATISTICS_FLAGS_VALID_DIRECTED_BYTES_XMIT
                                | NDIS_STATISTICS_FLAGS_VALID_MULTICAST_BYTES_XMIT
                                | NDIS_STATISTICS_FLAGS_VALID_BROADCAST_BYTES_XMIT,
                            ifHCInOctets: rx_bytes_unicast
                                + rx_bytes_multicast
                                + rx_bytes_broadcast,
                            ifHCInUcastOctets: rx_bytes_unicast,
                            ifHCInMulticastOctets: rx_bytes_multicast,
                            ifHCInBroadcastOctets: rx_bytes_broadcast,
                            ifHCInUcastPkts: adapter.rx_frames_unicast.load(Relaxed),
                            ifHCInMulticastPkts: adapter.rx_frames_multicast.load(Relaxed),
                            ifHCInBroadcastPkts: adapter.rx_frames_broadcast.load(Relaxed),
                            ifInErrors: adapter.rx_errors.load(Relaxed),
                            ifInDiscards: adapter.rx_discards.load(Relaxed),
                            ifHCOutOctets: tx_bytes_unicast
                                + tx_bytes_multicast
                                + tx_bytes_broadcast,
                            ifHCOutUcastOctets: tx_bytes_unicast,
                            ifHCOutMulticastOctets: tx_bytes_multicast,
                            ifHCOutBroadcastOctets: tx_bytes_broadcast,
                            ifHCOutUcastPkts: adapter.tx_frames_unicast.load(Relaxed),
                            ifHCOutMulticastPkts: adapter.tx_frames_multicast.load(Relaxed),
                            ifHCOutBroadcastPkts: adapter.tx_frames_broadcast.load(Relaxed),
                            ifOutErrors: adapter.tx_errors.load(Relaxed),
                            ifOutDiscards: adapter.tx_discards.load(Relaxed),
                        }
                    };
                    unsafe { write(request_data, result)? };
                    Ok(())
                }
                _ => Err(NDIS_STATUS_NOT_SUPPORTED.into()),
            }
        }
        NDIS_REQUEST_TYPE::NdisRequestSetInformation => {
            unsafe fn read<T>(
                request_data: &mut NDIS_OID_REQUEST_0_1,
                f: impl FnOnce(&T),
            ) -> Result<()> {
                let size = mem::size_of::<T>() as u32;
                if request_data.InformationBufferLength < size {
                    request_data.BytesNeeded = size;
                    return Err(NDIS_STATUS_INVALID_LENGTH.into());
                }
                f(&*request_data.InformationBuffer.cast());
                request_data.BytesRead = size;
                Ok(())
            }

            let request_data = unsafe { &mut (*oid_request).DATA.SET_INFORMATION };
            match request_data.Oid {
                OID_GEN_CURRENT_PACKET_FILTER => {
                    unsafe { read(request_data, |_: &u32| {})? };
                    Ok(())
                }
                OID_GEN_INTERRUPT_MODERATION => Err(NDIS_STATUS_INVALID_DATA.into()),
                _ => Err(NDIS_STATUS_NOT_SUPPORTED.into()),
            }
        }
        _ => Err(NDIS_STATUS_NOT_SUPPORTED.into()),
    })()
    .into()
}

extern "system" fn evt_send_net_buffer_lists(
    adapter_context: NDIS_HANDLE,
    nbl_chain: *mut NET_BUFFER_LIST,
    _port_number: NDIS_PORT_NUMBER,
    send_flags: u32,
) {
    let adapter = unsafe { Device::from_adapter_context(adapter_context).adapter() };
    let at_dispatch = (send_flags & NDIS_SEND_FLAGS_DISPATCH_LEVEL) != 0;
    let nbl_chain_tail = unsafe { NblChain::last(nbl_chain) };
    unsafe {
        adapter
            .tx_queue
            .lock_fast(at_dispatch)
            .enqueue_fast(nbl_chain, nbl_chain_tail)
    };
    adapter.tx_event.set();
}

extern "system" fn evt_return_net_buffer_lists(
    adapter_context: NDIS_HANDLE,
    nbl_chain: *mut NET_BUFFER_LIST,
    return_flags: u32,
) {
    let adapter = unsafe { Device::from_adapter_context(adapter_context).adapter() };
    let at_dispatch = (return_flags & NDIS_RETURN_FLAGS_DISPATCH_LEVEL) != 0;
    let nbl_iter = unsafe { LinkedIter::new(nbl_chain) }; // r.a
    for nbl in nbl_iter {
        let datagram = unsafe { *nbl.wsk_datagram_mut() };
        let socket = unsafe { adapter.socket.as_ref().unwrap_unchecked() };
        drop(socket.release(datagram));
    }
    let nbl_chain_tail = unsafe { NblChain::last(nbl_chain) }; // TODO
    unsafe {
        adapter
            .rx_queue
            .lock_fast(at_dispatch)
            .enqueue_fast(nbl_chain, nbl_chain_tail)
    };
}

extern "system" fn evt_cancel_send(_adapter_context: NDIS_HANDLE, _cancel_id: *mut c_void) {}

extern "system" fn evt_device_pnp_event_notify(
    _adapter_context: NDIS_HANDLE,
    _net_device_pnp_event: *mut NET_DEVICE_PNP_EVENT,
) {
}

extern "system" fn evt_shutdown(
    _adapter_context: NDIS_HANDLE,
    _shutdown_action: NDIS_SHUTDOWN_ACTION,
) {
}

extern "system" fn evt_cancel_oid_request(_adapter_context: NDIS_HANDLE, _request_id: *mut c_void) {
}

extern "system" fn evt_file_create(_device_object: *mut DEVICE_OBJECT, irp: *mut IRP) -> NTSTATUS {
    let status = STATUS_SUCCESS;
    unsafe { Request::complete(irp, status) };
    status
}

extern "system" fn evt_file_close(_device_object: *mut DEVICE_OBJECT, irp: *mut IRP) -> NTSTATUS {
    let status = STATUS_SUCCESS;
    unsafe { Request::complete(irp, status) };
    status
}

extern "system" fn evt_io_device_control(
    device_object: *mut DEVICE_OBJECT,
    irp: *mut IRP,
) -> NTSTATUS {
    let adapter = unsafe { Device::from_device_object(device_object).adapter_mut() };
    let irp_sp = unsafe { IoGetCurrentIrpStackLocation(irp) };
    let params = unsafe { &(*irp_sp).Parameters };
    let status = (|| match params.IoControlCode {
        nvnet_shared::IOCTL_VNET_SET_CONNECT_STATE => {
            let connected = unsafe { Request::retrieve_input_val::<bool>(irp)? };
            if unsafe { connected.read() } {
                adapter.connect()?;
            } else {
                adapter.disconnect()?;
            }
            Ok(())
        }
        nvnet_shared::IOCTL_VNET_SET_LOCAL_ENDPOINT => {
            let endpoint = unsafe { Request::retrieve_input_val::<SOCKADDR_IN6>(irp)? };
            unsafe { adapter.set_local_endpoint(endpoint.read()) };
            Ok(())
        }
        nvnet_shared::IOCTL_VNET_SET_REMOTE_ENDPOINT => {
            let endpoint = unsafe { Request::retrieve_input_val::<SOCKADDR_IN6>(irp)? };
            unsafe { adapter.set_remote_endpoint(endpoint.read()) };
            Ok(())
        }
        _ => Err(STATUS_NOT_SUPPORTED),
    })()
    .into();
    unsafe { Request::complete(irp, status) };
    status
}

extern "system" fn evt_recv_from(
    socket_context: *mut c_void,
    flags: u32,
    datagram_chain: *mut WSK_DATAGRAM_INDICATION,
) -> NTSTATUS {
    let adapter = unsafe { &*socket_context.cast::<Adapter>() };
    let at_dispatch = (flags & WSK_FLAG_AT_DISPATCH_LEVEL) != 0;
    let mut nbl_queue = MaybeUninit::uninit();
    unsafe { LinkedQueue::init(nbl_queue.as_mut_ptr()) };
    let nbl_queue = unsafe { nbl_queue.assume_init_mut() };
    let mut nbl_count = 0;
    let mut datagram_chain = unsafe { datagram_chain.as_mut() };
    while let Some(datagram) = datagram_chain {
        let datagram_next = unsafe { datagram.Next.as_mut() };
        datagram.Next = ptr::null_mut();
        let nbl = adapter.rx_queue.lock_fast(at_dispatch).dequeue();
        let nbl = unsafe { nbl.as_mut() };
        match nbl {
            None => {
                let socket = unsafe { adapter.socket.as_ref().unwrap_unchecked() };
                drop(socket.release(datagram));
                adapter.rx_discards.fetch_add(1, Relaxed);
            }
            Some(nbl) => {
                let WSK_BUF {
                    Mdl: mdl,
                    Offset: offset,
                    Length: length,
                } = datagram.Buffer;
                unsafe { *nbl.wsk_datagram_mut() = datagram };
                let nb = nbl.first_net_buffer;
                let nb = unsafe { &mut *nb };
                nb.current_mdl = mdl;
                nb.current_mdl_offset = offset;
                nb.data_length = length as u32;
                nb.mdl_chain = mdl;
                nb.data_offset = offset;
                unsafe { nbl_queue.enqueue(nbl) };
                nbl_count += 1;
                adapter.rx_bytes_unicast.fetch_add(length as u64, Relaxed);
                adapter.rx_frames_unicast.fetch_add(1, Relaxed);
            }
        }
        datagram_chain = datagram_next;
    }
    let nbl_chain = unsafe { nbl_queue.chain() };
    if !nbl_chain.is_null() {
        unsafe {
            NdisMIndicateReceiveNetBufferLists(
                adapter.handle,
                nbl_chain,
                NDIS_DEFAULT_PORT_NUMBER,
                nbl_count,
                if at_dispatch {
                    NDIS_RECEIVE_FLAGS_DISPATCH_LEVEL
                } else {
                    0
                },
            )
        };
    }
    STATUS_PENDING
}
