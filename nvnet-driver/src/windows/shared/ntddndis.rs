use crate::{
    windows::shared::{
        ifdef::{
            IF_MAX_PHYS_ADDRESS_LENGTH, NET_IF_MEDIA_CONNECT_STATE, NET_IF_MEDIA_DUPLEX_STATE,
        },
        ndis::{objectheader::NDIS_OBJECT_HEADER, oidtypes::NDIS_OID},
    },
    RTL_SIZEOF_THROUGH_FIELD,
};

// L190
pub const NDIS_OBJECT_TYPE_DEFAULT: u8 = 0x80;
pub const NDIS_OBJECT_TYPE_DEVICE_OBJECT_ATTRIBUTES: u8 = 0x85;
pub const NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS: u8 = 0x8A;
pub const NDIS_OBJECT_TYPE_STATUS_INDICATION: u8 = 0x98;
pub const NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES: u8 = 0x9E;
pub const NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES: u8 = 0x9F;

// L281
pub const NDIS_STATISTICS_FLAGS_VALID_DIRECTED_FRAMES_RCV: u32 = 0x00000001;
pub const NDIS_STATISTICS_FLAGS_VALID_MULTICAST_FRAMES_RCV: u32 = 0x00000002;
pub const NDIS_STATISTICS_FLAGS_VALID_BROADCAST_FRAMES_RCV: u32 = 0x00000004;
pub const NDIS_STATISTICS_FLAGS_VALID_BYTES_RCV: u32 = 0x00000008;
pub const NDIS_STATISTICS_FLAGS_VALID_RCV_DISCARDS: u32 = 0x00000010;
pub const NDIS_STATISTICS_FLAGS_VALID_RCV_ERROR: u32 = 0x00000020;
pub const NDIS_STATISTICS_FLAGS_VALID_DIRECTED_FRAMES_XMIT: u32 = 0x00000040;
pub const NDIS_STATISTICS_FLAGS_VALID_MULTICAST_FRAMES_XMIT: u32 = 0x00000080;
pub const NDIS_STATISTICS_FLAGS_VALID_BROADCAST_FRAMES_XMIT: u32 = 0x00000100;
pub const NDIS_STATISTICS_FLAGS_VALID_BYTES_XMIT: u32 = 0x00000200;
pub const NDIS_STATISTICS_FLAGS_VALID_XMIT_ERROR: u32 = 0x00000400;
pub const NDIS_STATISTICS_FLAGS_VALID_XMIT_DISCARDS: u32 = 0x00008000;
pub const NDIS_STATISTICS_FLAGS_VALID_DIRECTED_BYTES_RCV: u32 = 0x00010000;
pub const NDIS_STATISTICS_FLAGS_VALID_MULTICAST_BYTES_RCV: u32 = 0x00020000;
pub const NDIS_STATISTICS_FLAGS_VALID_BROADCAST_BYTES_RCV: u32 = 0x00040000;
pub const NDIS_STATISTICS_FLAGS_VALID_DIRECTED_BYTES_XMIT: u32 = 0x00080000;
pub const NDIS_STATISTICS_FLAGS_VALID_MULTICAST_BYTES_XMIT: u32 = 0x00100000;
pub const NDIS_STATISTICS_FLAGS_VALID_BROADCAST_BYTES_XMIT: u32 = 0x00200000;

// L301
pub const NDIS_STATISTICS_INFO_REVISION_1: u8 = 1;

c_type!(
    pub struct NDIS_STATISTICS_INFO {
        pub Header: NDIS_OBJECT_HEADER,
        pub SupportedStatistics: u32,
        pub ifInDiscards: u64,
        pub ifInErrors: u64,
        pub ifHCInOctets: u64,
        pub ifHCInUcastPkts: u64,
        pub ifHCInMulticastPkts: u64,
        pub ifHCInBroadcastPkts: u64,
        pub ifHCOutOctets: u64,
        pub ifHCOutUcastPkts: u64,
        pub ifHCOutMulticastPkts: u64,
        pub ifHCOutBroadcastPkts: u64,
        pub ifOutErrors: u64,
        pub ifOutDiscards: u64,
        pub ifHCInUcastOctets: u64,
        pub ifHCInMulticastOctets: u64,
        pub ifHCInBroadcastOctets: u64,
        pub ifHCOutUcastOctets: u64,
        pub ifHCOutMulticastOctets: u64,
        pub ifHCOutBroadcastOctets: u64,
    }
);

pub const NDIS_SIZEOF_STATISTICS_INFO_REVISION_1: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NDIS_STATISTICS_INFO, ifHCOutBroadcastOctets);

// L363
c_type!(
    pub enum NDIS_INTERRUPT_MODERATION {
        NdisInterruptModerationNotSupported = 1,
    }
);

pub const NDIS_INTERRUPT_MODERATION_PARAMETERS_REVISION_1: u8 = 1;

c_type!(
    pub struct NDIS_INTERRUPT_MODERATION_PARAMETERS {
        pub Header: NDIS_OBJECT_HEADER,
        pub Flags: u32,
        pub InterruptModeration: NDIS_INTERRUPT_MODERATION,
    }
);

pub const NDIS_SIZEOF_INTERRUPT_MODERATION_PARAMETERS_REVISION_1: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NDIS_INTERRUPT_MODERATION_PARAMETERS, InterruptModeration);

// L472
pub const OID_GEN_CURRENT_PACKET_FILTER: NDIS_OID = NDIS_OID(0x0001010E);
pub const OID_GEN_MAXIMUM_TOTAL_SIZE: NDIS_OID = NDIS_OID(0x00010111);
pub const OID_GEN_INTERRUPT_MODERATION: NDIS_OID = NDIS_OID(0x00010209);
pub const OID_GEN_XMIT_OK: NDIS_OID = NDIS_OID(0x00020101);
pub const OID_GEN_RCV_OK: NDIS_OID = NDIS_OID(0x00020102);
pub const OID_GEN_STATISTICS: NDIS_OID = NDIS_OID(0x00020106);

// L2258
c_type!(
    pub enum NDIS_MEDIUM {
        NdisMedium802_3 = 0,
    }
);

c_type!(
    pub enum NDIS_PHYSICAL_MEDIUM {
        NdisPhysicalMediumUnspecified = 0,
    }
);

pub const NDIS_PROTOCOL_ID_DEFAULT: u8 = 0x00;

// L2454
c_type!(
    pub enum NDIS_DEVICE_POWER_STATE;
);

// L2485
c_type!(
    pub struct NDIS_PNP_CAPABILITIES;
);

// L2718
pub const NDIS_PACKET_TYPE_DIRECTED: u32 = 0x00000001;
pub const NDIS_PACKET_TYPE_MULTICAST: u32 = 0x00000002;
pub const NDIS_PACKET_TYPE_ALL_MULTICAST: u32 = 0x00000004;
pub const NDIS_PACKET_TYPE_BROADCAST: u32 = 0x00000008;
pub const NDIS_PACKET_TYPE_PROMISCUOUS: u32 = 0x00000020;

// L2762
pub const NDIS_MAC_OPTION_NO_LOOPBACK: u32 = 0x00000008;

// L2817
pub const NDIS_MAX_PHYS_ADDRESS_LENGTH: usize = IF_MAX_PHYS_ADDRESS_LENGTH;

pub type NDIS_MEDIA_CONNECT_STATE = NET_IF_MEDIA_CONNECT_STATE;

pub type NDIS_MEDIA_DUPLEX_STATE = NET_IF_MEDIA_DUPLEX_STATE;

c_type!(
    pub enum NDIS_SUPPORTED_PAUSE_FUNCTIONS {
        NdisPauseFunctionsUnsupported = 0,
    }
);

pub const NDIS_LINK_STATE_REVISION_1: u8 = 1;

c_type!(
    pub struct NDIS_LINK_STATE {
        pub Header: NDIS_OBJECT_HEADER,
        pub MediaConnectState: NDIS_MEDIA_CONNECT_STATE,
        pub MediaDuplexState: NDIS_MEDIA_DUPLEX_STATE,
        pub XmitLinkSpeed: u64,
        pub RcvLinkSpeed: u64,
        pub PauseFunctions: NDIS_SUPPORTED_PAUSE_FUNCTIONS,
        pub AutoNegotiationFlags: u32,
    }
);

pub const NDIS_SIZEOF_LINK_STATE_REVISION_1: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NDIS_LINK_STATE, AutoNegotiationFlags);

// L4275
pub const NDIS_PM_CAPABILITIES_REVISION_2: u8 = 2;

c_type!(
    pub struct NDIS_PM_CAPABILITIES {
        pub Header: NDIS_OBJECT_HEADER,
        pub Flags: u32,
        pub SupportedWoLPacketPatterns: u32,
        pub NumTotalWoLPatterns: u32,
        pub MaxWoLPatternSize: u32,
        pub MaxWoLPatternOffset: u32,
        pub MaxWoLPacketSaveBuffer: u32,
        pub SupportedProtocolOffloads: u32,
        pub NumArpOffloadIPv4Addresses: u32,
        pub NumNSOffloadIPv6Addresses: u32,
        pub MinMagicPacketWakeUp: NDIS_DEVICE_POWER_STATE,
        pub MinPatternWakeUp: NDIS_DEVICE_POWER_STATE,
        pub MinLinkChangeWakeUp: NDIS_DEVICE_POWER_STATE,
        pub SupportedWakeUpEvents: u32,
        pub MediaSpecificWakeUpEvents: u32,
    }
);

pub const NDIS_SIZEOF_NDIS_PM_CAPABILITIES_REVISION_2: u16 =
    RTL_SIZEOF_THROUGH_FIELD!(NDIS_PM_CAPABILITIES, MediaSpecificWakeUpEvents);

// L5556
c_type!(
    pub struct NDIS_RECEIVE_SCALE_CAPABILITIES;
);
