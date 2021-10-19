use core::ffi::c_void;

use crate::{
    windows::{
        km::{
            ndis::{nbl::NET_BUFFER_LIST, oidrequest::NDIS_OID_REQUEST},
            wdm::{DEVICE_OBJECT, DRIVER_OBJECT, INTERFACE_TYPE, PDRIVER_DISPATCH},
        },
        shared::{
            guiddef::GUID,
            ifdef::{
                NET_IFTYPE, NET_IF_ACCESS_TYPE, NET_IF_CONNECTION_TYPE, NET_IF_DIRECTION_TYPE,
            },
            ndis::{
                ndisport::NDIS_PORT_NUMBER,
                objectheader::NDIS_OBJECT_HEADER,
                oidtypes::NDIS_OID,
                types::{NDIS_HANDLE, NDIS_STATUS},
            },
            ntddndis::{
                NDIS_MAX_PHYS_ADDRESS_LENGTH, NDIS_MEDIA_CONNECT_STATE, NDIS_MEDIA_DUPLEX_STATE,
                NDIS_MEDIUM, NDIS_PHYSICAL_MEDIUM, NDIS_PM_CAPABILITIES, NDIS_PNP_CAPABILITIES,
                NDIS_RECEIVE_SCALE_CAPABILITIES, NDIS_SUPPORTED_PAUSE_FUNCTIONS,
            },
            ntdef::UNICODE_STRING,
        },
    },
    RTL_SIZEOF_THROUGH_FIELD,
};

pub mod nbl;
pub mod nblapi;
pub mod nblreceive;
pub mod nblsend;
pub mod oidrequest;

#[allow(dead_code)]
pub const NDIS_MINIPORT_DRIVER: () = ();
#[allow(dead_code)]
pub const NDIS630_MINIPORT: () = ();

// L231
pub type NDIS_STRING = UNICODE_STRING;

// L337
pub const NDIS_MINIPORT_MAJOR_VERSION: u8 = 6;
pub const NDIS_MINIPORT_MINOR_VERSION: u8 = 30;

// L1198
c_type!(
    pub enum NDIS_INTERFACE_TYPE {
        NdisInterfaceInternal = INTERFACE_TYPE::Internal.0,
    }
);

// L1888
pub const NDIS_STATUS_LINK_STATE: NDIS_STATUS = NDIS_STATUS(0x40010017);

// L4116
c_type!(
    pub type SET_OPTIONS_HANDLER = fn() -> !;
);

// L4401
pub const NDIS_STATUS_INDICATION_REVISION_1: u8 = 1;

c_type!(
    pub struct NDIS_STATUS_INDICATION {
        pub Header: NDIS_OBJECT_HEADER,
        pub SourceHandle: NDIS_HANDLE,
        pub PortNumber: NDIS_PORT_NUMBER,
        pub StatusCode: NDIS_STATUS,
        pub Flags: u32,
        pub DestinationHandle: NDIS_HANDLE,
        pub RequestId: *mut c_void,
        pub StatusBuffer: *mut c_void,
        pub StatusBufferSize: u32,
        pub Guid: GUID,
        pub NdisReserved: [*mut c_void; 4],
    }
);

pub const NDIS_SIZEOF_STATUS_INDICATION_REVISION_1: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NDIS_STATUS_INDICATION, NdisReserved);

// L4709
pub const NDIS_DEVICE_OBJECT_ATTRIBUTES_REVISION_1: u8 = 1;

c_type!(
    pub struct NDIS_DEVICE_OBJECT_ATTRIBUTES {
        pub Header: NDIS_OBJECT_HEADER,
        pub DeviceName: *mut NDIS_STRING,
        pub SymbolicName: *mut NDIS_STRING,
        pub MajorFunctions: *mut PDRIVER_DISPATCH,
        pub ExtensionSize: u32,
        pub DefaultSDDLString: *const UNICODE_STRING,
        pub DeviceClassGuid: *const GUID,
    }
);

pub const NDIS_SIZEOF_DEVICE_OBJECT_ATTRIBUTES_REVISION_1: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NDIS_DEVICE_OBJECT_ATTRIBUTES, DeviceClassGuid);

extern "system" {
    // #[irql_requires(PASSIVE_LEVEL)]
    pub fn NdisRegisterDeviceEx(
        NdisHandle: NDIS_HANDLE,
        DeviceObjectAttributes: *mut NDIS_DEVICE_OBJECT_ATTRIBUTES,
        pDeviceObject: *mut *mut DEVICE_OBJECT,
        NdisDeviceHandle: *mut NDIS_HANDLE,
    ) -> NDIS_STATUS;

    // #[irql_requires(PASSIVE_LEVEL)]
    pub fn NdisDeregisterDeviceEx(NdisDeviceHandle: NDIS_HANDLE);

    // #[irql_requires_max(HIGH_LEVEL)]
    pub fn NdisGetDeviceReservedExtension(DeviceObject: *mut DEVICE_OBJECT) -> *mut c_void;
}

// L10129
c_type!(
    pub struct NDIS_MINIPORT_INIT_PARAMETERS;
);

c_type!(
    pub struct NDIS_MINIPORT_RESTART_PARAMETERS;
);

pub const NDIS_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES_REVISION_2: u8 = 2;

c_type!(
    pub struct NDIS_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES {
        pub Header: NDIS_OBJECT_HEADER,
        pub MiniportAdapterContext: NDIS_HANDLE,
        pub AttributeFlags: u32,
        pub CheckForHangTimeInSeconds: u32,
        pub InterfaceType: NDIS_INTERFACE_TYPE,
    }
);

pub const NDIS_SIZEOF_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES_REVISION_2: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NDIS_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES, InterfaceType);

pub const NDIS_STATISTICS_XMIT_ERROR_SUPPORTED: u32 = 0x00000004;
pub const NDIS_STATISTICS_RCV_ERROR_SUPPORTED: u32 = 0x00000008;
pub const NDIS_STATISTICS_DIRECTED_BYTES_XMIT_SUPPORTED: u32 = 0x00000020;
pub const NDIS_STATISTICS_DIRECTED_FRAMES_XMIT_SUPPORTED: u32 = 0x00000040;
pub const NDIS_STATISTICS_MULTICAST_BYTES_XMIT_SUPPORTED: u32 = 0x00000080;
pub const NDIS_STATISTICS_MULTICAST_FRAMES_XMIT_SUPPORTED: u32 = 0x00000100;
pub const NDIS_STATISTICS_BROADCAST_BYTES_XMIT_SUPPORTED: u32 = 0x00000200;
pub const NDIS_STATISTICS_BROADCAST_FRAMES_XMIT_SUPPORTED: u32 = 0x00000400;
pub const NDIS_STATISTICS_DIRECTED_BYTES_RCV_SUPPORTED: u32 = 0x00000800;
pub const NDIS_STATISTICS_DIRECTED_FRAMES_RCV_SUPPORTED: u32 = 0x00001000;
pub const NDIS_STATISTICS_MULTICAST_BYTES_RCV_SUPPORTED: u32 = 0x00002000;
pub const NDIS_STATISTICS_MULTICAST_FRAMES_RCV_SUPPORTED: u32 = 0x00004000;
pub const NDIS_STATISTICS_BROADCAST_BYTES_RCV_SUPPORTED: u32 = 0x00008000;
pub const NDIS_STATISTICS_BROADCAST_FRAMES_RCV_SUPPORTED: u32 = 0x00010000;
pub const NDIS_STATISTICS_BYTES_RCV_SUPPORTED: u32 = 0x00080000;
pub const NDIS_STATISTICS_BYTES_XMIT_SUPPORTED: u32 = 0x00100000;
pub const NDIS_STATISTICS_RCV_DISCARDS_SUPPORTED: u32 = 0x00200000;
pub const NDIS_STATISTICS_XMIT_DISCARDS_SUPPORTED: u32 = 0x08000000;

pub const NDIS_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES_REVISION_2: u8 = 2;

c_type!(
    pub struct NDIS_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES {
        pub Header: NDIS_OBJECT_HEADER,
        pub Flags: u32,
        pub MediaType: NDIS_MEDIUM,
        pub PhysicalMediumType: NDIS_PHYSICAL_MEDIUM,
        pub MtuSize: u32,
        pub MaxXmitLinkSpeed: u64,
        pub XmitLinkSpeed: u64,
        pub MaxRcvLinkSpeed: u64,
        pub RcvLinkSpeed: u64,
        pub MediaConnectState: NDIS_MEDIA_CONNECT_STATE,
        pub MediaDuplexState: NDIS_MEDIA_DUPLEX_STATE,
        pub LookaheadSize: u32,
        pub PowerManagementCapabilities: *mut NDIS_PNP_CAPABILITIES,
        pub MacOptions: u32,
        pub SupportedPacketFilters: u32,
        pub MaxMulticastListSize: u32,
        pub MacAddressLength: u16,
        pub PermanentMacAddress: [u8; NDIS_MAX_PHYS_ADDRESS_LENGTH],
        pub CurrentMacAddress: [u8; NDIS_MAX_PHYS_ADDRESS_LENGTH],
        pub RecvScaleCapabilities: *mut NDIS_RECEIVE_SCALE_CAPABILITIES,
        pub AccessType: NET_IF_ACCESS_TYPE,
        pub DirectionType: NET_IF_DIRECTION_TYPE,
        pub ConnectionType: NET_IF_CONNECTION_TYPE,
        pub IfType: NET_IFTYPE,
        pub IfConnectorPresent: bool,
        pub SupportedStatistics: u32,
        pub SupportedPauseFunctions: NDIS_SUPPORTED_PAUSE_FUNCTIONS,
        pub DataBackFillSize: u32,
        pub ContextBackFillSize: u32,
        pub SupportedOidList: *mut NDIS_OID,
        pub SupportedOidListLength: u32,
        pub AutoNegotiationFlags: u32,
        pub PowerManagementCapabilitiesEx: *mut NDIS_PM_CAPABILITIES,
    }
);

pub const NDIS_SIZEOF_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES_REVISION_2: u16 = RTL_SIZEOF_THROUGH_FIELD!(
    NDIS_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES,
    PowerManagementCapabilitiesEx
);

// L10520
c_type!(
    pub struct NDIS_MINIPORT_ADAPTER_ATTRIBUTES;
);

extern "system" {
    // #[irql_requires(PASSIVE_LEVEL)]
    pub fn NdisMSetMiniportAttributes(
        NdisMiniportHandle: NDIS_HANDLE,
        MiniportAttributes: *mut NDIS_MINIPORT_ADAPTER_ATTRIBUTES,
    ) -> NDIS_STATUS;
}

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    pub type MINIPORT_INITIALIZE_HANDLER = fn(
        NdisMiniportHandle: NDIS_HANDLE,
        MiniportDriverContext: NDIS_HANDLE,
        MiniportInitParameters: *mut NDIS_MINIPORT_INIT_PARAMETERS,
    ) -> NDIS_STATUS;
);

c_type!(
    pub enum NDIS_HALT_ACTION;
);

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    pub type MINIPORT_HALT_HANDLER =
        fn(MiniportAdapterContext: NDIS_HANDLE, HaltAction: NDIS_HALT_ACTION);
);

c_type!(
    pub struct NDIS_MINIPORT_PAUSE_PARAMETERS;
);

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    pub type MINIPORT_PAUSE_HANDLER = fn(
        MiniportAdapterContext: NDIS_HANDLE,
        PauseParameters: *mut NDIS_MINIPORT_PAUSE_PARAMETERS,
    ) -> NDIS_STATUS;
);

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    pub type MINIPORT_RESTART_HANDLER = fn(
        MiniportAdapterContext: NDIS_HANDLE,
        RestartParameters: *mut NDIS_MINIPORT_RESTART_PARAMETERS,
    ) -> NDIS_STATUS;
);

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    pub type MINIPORT_OID_REQUEST_HANDLER =
        fn(MiniportAdapterContext: NDIS_HANDLE, OidRequest: *mut NDIS_OID_REQUEST) -> NDIS_STATUS;
);

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    pub type MINIPORT_DRIVER_UNLOAD = fn(DriverObject: *mut DRIVER_OBJECT);
);

c_type!(
    pub enum NDIS_SHUTDOWN_ACTION;
);

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    // ...
    pub type MINIPORT_SHUTDOWN_HANDLER =
        fn(MiniportAdapterContext: NDIS_HANDLE, ShutdownAction: NDIS_SHUTDOWN_ACTION);
);

c_type!(
    pub struct NET_DEVICE_PNP_EVENT;
);

c_type!(
    // #[irql_requires(PASSIVE_LEVEL)]
    pub type MINIPORT_DEVICE_PNP_EVENT_NOTIFY_HANDLER =
        fn(MiniportAdapterContext: NDIS_HANDLE, NetDevicePnPEvent: *mut NET_DEVICE_PNP_EVENT);
);

c_type!(
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub type MINIPORT_CANCEL_SEND_HANDLER =
        fn(MiniportAdapterContext: NDIS_HANDLE, CancelId: *mut c_void);
);

c_type!(
    pub type MINIPORT_CHECK_FOR_HANG_HANDLER = fn() -> !;
);

c_type!(
    pub type MINIPORT_RESET_HANDLER = fn() -> !;
);

c_type!(
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub type MINIPORT_CANCEL_OID_REQUEST_HANDLER =
        fn(MiniportAdapterContext: NDIS_HANDLE, RequestId: *mut c_void);
);

c_type!(
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub type MINIPORT_SEND_NET_BUFFER_LISTS_HANDLER = fn(
        MiniportAdapterContext: NDIS_HANDLE,
        NetBufferList: *mut NET_BUFFER_LIST,
        PortNumber: NDIS_PORT_NUMBER,
        SendFlags: u32,
    );
);

c_type!(
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub type MINIPORT_RETURN_NET_BUFFER_LISTS_HANDLER = fn(
        MiniportAdapterContext: NDIS_HANDLE,
        NetBufferLists: *mut NET_BUFFER_LIST,
        ReturnFlags: u32,
    );
);

c_type!(
    pub type MINIPORT_DIRECT_OID_REQUEST_HANDLER = fn() -> !;
);

c_type!(
    pub type MINIPORT_CANCEL_DIRECT_OID_REQUEST_HANDLER = fn() -> !;
);

// L10838
pub const NDIS_MINIPORT_DRIVER_CHARACTERISTICS_REVISION_2: u8 = 2;

c_type!(
    pub struct NDIS_MINIPORT_DRIVER_CHARACTERISTICS {
        pub Header: NDIS_OBJECT_HEADER,
        pub MajorNdisVersion: u8,
        pub MinorNdisVersion: u8,
        pub MajorDriverVersion: u8,
        pub MinorDriverVersion: u8,
        pub Flags: u32,
        pub SetOptionsHandler: SET_OPTIONS_HANDLER,
        pub InitializeHandlerEx: MINIPORT_INITIALIZE_HANDLER,
        pub HaltHandlerEx: MINIPORT_HALT_HANDLER,
        pub UnloadHandler: MINIPORT_DRIVER_UNLOAD,
        pub PauseHandler: MINIPORT_PAUSE_HANDLER,
        pub RestartHandler: MINIPORT_RESTART_HANDLER,
        pub OidRequestHandler: MINIPORT_OID_REQUEST_HANDLER,
        pub SendNetBufferListsHandler: MINIPORT_SEND_NET_BUFFER_LISTS_HANDLER,
        pub ReturnNetBufferListsHandler: MINIPORT_RETURN_NET_BUFFER_LISTS_HANDLER,
        pub CancelSendHandler: MINIPORT_CANCEL_SEND_HANDLER,
        pub CheckForHangHandlerEx: MINIPORT_CHECK_FOR_HANG_HANDLER,
        pub ResetHandlerEx: MINIPORT_RESET_HANDLER,
        pub DevicePnPEventNotifyHandler: MINIPORT_DEVICE_PNP_EVENT_NOTIFY_HANDLER,
        pub ShutdownHandlerEx: MINIPORT_SHUTDOWN_HANDLER,
        pub CancelOidRequestHandler: MINIPORT_CANCEL_OID_REQUEST_HANDLER,
        pub DirectOidRequestHandler: MINIPORT_DIRECT_OID_REQUEST_HANDLER,
        pub CancelDirectOidRequestHandler: MINIPORT_CANCEL_DIRECT_OID_REQUEST_HANDLER,
    }
);

pub const NDIS_SIZEOF_MINIPORT_DRIVER_CHARACTERISTICS_REVISION_2: u16 = RTL_SIZEOF_THROUGH_FIELD!(
    NDIS_MINIPORT_DRIVER_CHARACTERISTICS,
    CancelDirectOidRequestHandler
);

// L10915
extern "system" {
    pub fn NdisMIndicateStatusEx(
        MiniportAdapterHandle: NDIS_HANDLE,
        StatusIndication: *mut NDIS_STATUS_INDICATION,
    );
}

// L12430
extern "system" {
    // #[irql_requires(PASSIVE_LEVEL)]
    pub fn NdisMRegisterMiniportDriver(
        DriverObject: *mut DRIVER_OBJECT,
        RegistryPath: *mut UNICODE_STRING,
        MiniportDriverContext: NDIS_HANDLE,
        MiniportDriverCharacteristics: *mut NDIS_MINIPORT_DRIVER_CHARACTERISTICS,
        NdisMiniportDriverHandle: *mut NDIS_HANDLE,
    ) -> NDIS_STATUS;

    // #[irql_requires(PASSIVE_LEVEL)]
    pub fn NdisMDeregisterMiniportDriver(NdisMiniportDriverHandle: NDIS_HANDLE);

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn NdisMSendNetBufferListsComplete(
        MiniportAdapterHandle: NDIS_HANDLE,
        NetBufferList: *mut NET_BUFFER_LIST,
        SendCompleteFlags: u32,
    );

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn NdisMIndicateReceiveNetBufferLists(
        MiniportAdapterHandle: NDIS_HANDLE,
        NetBufferList: *mut NET_BUFFER_LIST,
        PortNumber: NDIS_PORT_NUMBER,
        NumberOfNetBufferLists: u32,
        ReceiveFlags: u32,
    );
}
