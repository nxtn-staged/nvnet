use core::{default::default, mem};

use libnveth_macros::*;

use crate::windows::{
    km::{
        netcx::kmdf::adapter::{
            netadaptercxtypes::{NETADAPTER, NETADAPTER_INIT},
            netrxqueue::NETRXQUEUE_INIT,
            nettxqueue::NETTXQUEUE_INIT,
        },
        wdm::NODE_REQUIREMENT,
    },
    shared::{
        ifdef::{NET_IF_MEDIA_CONNECT_STATE, NET_IF_MEDIA_DUPLEX_STATE},
        ntddndis::{
            NDIS_IF_PHYSICAL_ADDRESS, NDIS_PACKET_TYPE_ALL_MULTICAST, NDIS_PACKET_TYPE_BROADCAST,
            NDIS_PACKET_TYPE_DIRECTED, NDIS_PACKET_TYPE_MULTICAST, NDIS_PACKET_TYPE_PROMISCUOUS,
            NDIS_SUPPORTED_PAUSE_FUNCTIONS,
        },
        ntdef::{NTSTATUS, PHYSICAL_ADDRESS},
    },
    wdf::kmdf::{
        wdfobject::WDF_OBJECT_ATTRIBUTES,
        wdftypes::{WDFDEVICE, WDFDMAENABLER, WDF_TRI_STATE},
    },
};

c_type!(
    pub enum NET_ADAPTER_PAUSE_FUNCTION_TYPE {
        NetAdapterPauseFunctionTypeUnsupported =
            NDIS_SUPPORTED_PAUSE_FUNCTIONS::NdisPauseFunctionsUnsupported as _,
    }
);

c_type!(
    pub enum NET_ADAPTER_AUTO_NEGOTIATION_FLAGS {
        NetAdapterAutoNegotiationFlagNone = 0,
    }
);

c_type!(
    pub enum NET_MEMORY_MAPPING_REQUIREMENT {
        NetMemoryMappingRequirementNone = 0,
    }
);

c_type!(
    pub type PFN_NET_ADAPTER_CREATE_TXQUEUE =
        fn(adapter: NETADAPTER, tx_queue_init: *mut NETTXQUEUE_INIT) -> NTSTATUS;
);

c_type!(
    pub type PFN_NET_ADAPTER_CREATE_RXQUEUE =
        fn(adapter: NETADAPTER, rx_queue_init: *mut NETRXQUEUE_INIT) -> NTSTATUS;
);

pub type NET_ADAPTER_LINK_LAYER_ADDRESS = NDIS_IF_PHYSICAL_ADDRESS;

c_type!(
    pub struct NET_ADAPTER_LINK_LAYER_CAPABILITIES {
        pub size: u32,
        pub max_tx_link_speed: u64,
        pub max_rx_link_speed: u64,
    }
);

pub fn NET_ADAPTER_LINK_LAYER_CAPABILITIES_INIT(
    max_tx_link_speed: u64,
    max_rx_link_speed: u64,
) -> NET_ADAPTER_LINK_LAYER_CAPABILITIES {
    NET_ADAPTER_LINK_LAYER_CAPABILITIES {
        size: mem::size_of::<NET_ADAPTER_LINK_LAYER_CAPABILITIES>() as _,
        max_tx_link_speed,
        max_rx_link_speed,
    }
}

c_type!(
    pub struct NET_ADAPTER_DMA_CAPABILITIES {
        pub size: u32,
        pub dma_enabler: WDFDMAENABLER,
        pub maximum_physical_address: PHYSICAL_ADDRESS,
        pub cache_enabled: WDF_TRI_STATE,
        pub preferred_node: NODE_REQUIREMENT,
    }
);

pub const NET_ADAPTER_FRAGMENT_DEFAULT_ALIGNMENT: usize = 1;

c_type!(
    pub enum NET_RX_FRAGMENT_BUFFER_ALLOCATION_MODE {
        NetRxFragmentBufferAllocationModeSystem = 0,
    }
);

c_type!(
    pub enum NET_RX_FRAGMENT_BUFFER_ATTACHMENT_MODE {
        NetRxFragmentBufferAttachmentModeSystem = 0,
    }
);

c_type!(
    pub struct NET_ADAPTER_RX_CAPABILITIES {
        pub size: u32,
        pub allocation_mode: NET_RX_FRAGMENT_BUFFER_ALLOCATION_MODE,
        pub attachment_mode: NET_RX_FRAGMENT_BUFFER_ATTACHMENT_MODE,
        pub fragment_ring_number_of_elements_hint: u32,
        pub maximum_frame_size: usize,
        pub maximum_number_of_queues: usize,
        pub u: NET_ADAPTER_RX_CAPABILITIES_u,
    }
);

c_type!(
    pub union NET_ADAPTER_RX_CAPABILITIES_u {
        pub s: NET_ADAPTER_RX_CAPABILITIES_u_s,
        pub s2: NET_ADAPTER_RX_CAPABILITIES_u_s2,
    }
);

c_type!(
    pub struct NET_ADAPTER_RX_CAPABILITIES_u_s;
);

c_type!(
    pub struct NET_ADAPTER_RX_CAPABILITIES_u_s2 {
        pub mapping_requirement: NET_MEMORY_MAPPING_REQUIREMENT,
        pub fragment_buffer_alignment: usize,
        pub dma_capabilities: *mut NET_ADAPTER_DMA_CAPABILITIES,
    }
);

pub fn NET_ADAPTER_RX_CAPABILITIES_INIT_SYSTEM_MANAGED(
    maximum_frame_size: usize,
    maximum_number_of_queues: usize,
) -> NET_ADAPTER_RX_CAPABILITIES {
    NET_ADAPTER_RX_CAPABILITIES {
        size: mem::size_of::<NET_ADAPTER_RX_CAPABILITIES>() as _,
        maximum_frame_size,
        maximum_number_of_queues,
        allocation_mode:
            NET_RX_FRAGMENT_BUFFER_ALLOCATION_MODE::NetRxFragmentBufferAllocationModeSystem,
        attachment_mode:
            NET_RX_FRAGMENT_BUFFER_ATTACHMENT_MODE::NetRxFragmentBufferAttachmentModeSystem,
        u: NET_ADAPTER_RX_CAPABILITIES_u {
            s2: NET_ADAPTER_RX_CAPABILITIES_u_s2 {
                fragment_buffer_alignment: NET_ADAPTER_FRAGMENT_DEFAULT_ALIGNMENT,
                mapping_requirement:
                    NET_MEMORY_MAPPING_REQUIREMENT::NetMemoryMappingRequirementNone,
                ..default()
            },
        },
        ..default()
    }
}

c_type!(
    pub struct NET_ADAPTER_TX_CAPABILITIES {
        pub size: u32,
        pub mapping_requirement: NET_MEMORY_MAPPING_REQUIREMENT,
        pub payload_backfill: usize,
        pub maximum_number_of_fragments: usize,
        pub fragment_buffer_alignment: usize,
        pub fragment_ring_number_of_elements_hint: usize,
        pub maximum_number_of_queues: usize,
        pub dma_capabilities: *mut NET_ADAPTER_DMA_CAPABILITIES,
    }
);

pub fn NET_ADAPTER_TX_CAPABILITIES_INIT(
    maximum_number_of_queues: usize,
) -> NET_ADAPTER_TX_CAPABILITIES {
    NET_ADAPTER_TX_CAPABILITIES {
        size: mem::size_of::<NET_ADAPTER_TX_CAPABILITIES>() as _,
        fragment_buffer_alignment: NET_ADAPTER_FRAGMENT_DEFAULT_ALIGNMENT,
        maximum_number_of_queues,
        maximum_number_of_fragments: -1 as _,
        ..default()
    }
}

c_type!(
    pub struct NET_ADAPTER_LINK_STATE {
        pub size: u32,
        pub tx_link_speed: u64,
        pub rx_link_speed: u64,
        pub media_connect_state: NET_IF_MEDIA_CONNECT_STATE,
        pub media_duplex_state: NET_IF_MEDIA_DUPLEX_STATE,
        pub supported_pause_functions: NET_ADAPTER_PAUSE_FUNCTION_TYPE,
        pub auto_negotiation_flags: NET_ADAPTER_AUTO_NEGOTIATION_FLAGS,
    }
);

pub fn NET_ADAPTER_LINK_STATE_INIT(
    link_speed: u64,
    media_connect_state: NET_IF_MEDIA_CONNECT_STATE,
    media_duplex_state: NET_IF_MEDIA_DUPLEX_STATE,
    supported_pause_functions: NET_ADAPTER_PAUSE_FUNCTION_TYPE,
    auto_negotiation_flags: NET_ADAPTER_AUTO_NEGOTIATION_FLAGS,
) -> NET_ADAPTER_LINK_STATE {
    NET_ADAPTER_LINK_STATE {
        size: mem::size_of::<NET_ADAPTER_LINK_STATE>() as _,
        tx_link_speed: link_speed,
        rx_link_speed: link_speed,
        media_connect_state,
        media_duplex_state,
        supported_pause_functions,
        auto_negotiation_flags,
    }
}

c_type!(
    pub struct NET_ADAPTER_DATAPATH_CALLBACKS {
        pub size: u32,
        pub evt_adapter_create_tx_queue: PFN_NET_ADAPTER_CREATE_TXQUEUE,
        pub evt_adapter_create_rx_queue: PFN_NET_ADAPTER_CREATE_RXQUEUE,
    }
);

pub fn NET_ADAPTER_DATAPATH_CALLBACKS_INIT(
    evt_adapter_create_tx_queue: PFN_NET_ADAPTER_CREATE_TXQUEUE,
    evt_adapter_create_rx_queue: PFN_NET_ADAPTER_CREATE_RXQUEUE,
) -> NET_ADAPTER_DATAPATH_CALLBACKS {
    NET_ADAPTER_DATAPATH_CALLBACKS {
        size: mem::size_of::<NET_ADAPTER_DATAPATH_CALLBACKS>() as _,
        evt_adapter_create_tx_queue,
        evt_adapter_create_rx_queue,
    }
}

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetAdapterInitAllocate(device: WDFDEVICE) -> *mut NETADAPTER_INIT {
        NetAdapterInitAllocateTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetAdapterInitFree(adapter_init: *mut NETADAPTER_INIT) -> () {
        NetAdapterInitFreeTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetAdapterInitSetDatapathCallbacks(
        adapter_init: *mut NETADAPTER_INIT,
        datapath_callbacks: *const NET_ADAPTER_DATAPATH_CALLBACKS,
    ) -> () {
        NetAdapterInitSetDatapathCallbacksTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetAdapterCreate(
        adapter_init: *mut NETADAPTER_INIT,
        adapter_attributes: *const WDF_OBJECT_ATTRIBUTES,
        adapter: *mut NETADAPTER,
    ) -> NTSTATUS {
        NetAdapterCreateTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetAdapterStart(adapter: NETADAPTER) -> NTSTATUS {
        NetAdapterStartTableIndex
    }
);

net_fn!(
    #[irql_requires(PASSIVE_LEVEL)]
    pub fn NetAdapterSetLinkLayerCapabilities(
        adapter: NETADAPTER,
        link_layer_capabilities: *const NET_ADAPTER_LINK_LAYER_CAPABILITIES,
    ) -> () {
        NetAdapterSetLinkLayerCapabilitiesTableIndex
    }
);

net_fn!(
    #[irql_requires(PASSIVE_LEVEL)]
    pub fn NetAdapterSetLinkLayerMtuSize(adapter: NETADAPTER, mtu_size: u32) -> () {
        NetAdapterSetLinkLayerMtuSizeTableIndex
    }
);

net_fn!(
    #[irql_requires(PASSIVE_LEVEL)]
    pub fn NetAdapterSetDataPathCapabilities(
        adapter: NETADAPTER,
        tx_capabilities: *const NET_ADAPTER_TX_CAPABILITIES,
        rx_capabilities: *const NET_ADAPTER_RX_CAPABILITIES,
    ) -> () {
        NetAdapterSetDataPathCapabilitiesTableIndex
    }
);

net_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn NetAdapterSetLinkState(adapter: NETADAPTER, state: *const NET_ADAPTER_LINK_STATE) -> () {
        NetAdapterSetLinkStateTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetAdapterSetPermanentLinkLayerAddress(
        adapter: NETADAPTER,
        link_layer_address: *const NET_ADAPTER_LINK_LAYER_ADDRESS,
    ) -> () {
        NetAdapterSetPermanentLinkLayerAddressTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetAdapterSetCurrentLinkLayerAddress(
        adapter: NETADAPTER,
        link_layer_address: *const NET_ADAPTER_LINK_LAYER_ADDRESS,
    ) -> () {
        NetAdapterSetCurrentLinkLayerAddressTableIndex
    }
);

c_type!(
    #[flags]
    pub enum NET_PACKET_FILTER_FLAGS {
        NetPacketFilterFlagDirected = NDIS_PACKET_TYPE_DIRECTED,
        NetPacketFilterFlagMulticast = NDIS_PACKET_TYPE_MULTICAST,
        NetPacketFilterFlagAllMulticast = NDIS_PACKET_TYPE_ALL_MULTICAST,
        NetPacketFilterFlagBroadcast = NDIS_PACKET_TYPE_BROADCAST,
        NetPacketFilterFlagPromiscuous = NDIS_PACKET_TYPE_PROMISCUOUS,
    }
);

c_type!(
    pub type PFN_NET_ADAPTER_SET_PACKET_FILTER =
        fn(adapter: NETADAPTER, packet_filter: NET_PACKET_FILTER_FLAGS) -> ();
);

c_type!(
    pub struct NET_ADAPTER_PACKET_FILTER_CAPABILITIES {
        pub size: u32,
        pub supported_packet_filters: NET_PACKET_FILTER_FLAGS,
        pub evt_set_packet_filter: PFN_NET_ADAPTER_SET_PACKET_FILTER,
    }
);

pub fn NET_ADAPTER_PACKET_FILTER_CAPABILITIES_INIT(
    filters: NET_PACKET_FILTER_FLAGS,
    evt_set_packet_filter: PFN_NET_ADAPTER_SET_PACKET_FILTER,
) -> NET_ADAPTER_PACKET_FILTER_CAPABILITIES {
    NET_ADAPTER_PACKET_FILTER_CAPABILITIES {
        size: mem::size_of::<NET_ADAPTER_PACKET_FILTER_CAPABILITIES>() as _,
        supported_packet_filters: filters,
        evt_set_packet_filter,
    }
}

net_fn!(
    #[irql_requires(PASSIVE_LEVEL)]
    pub fn NetAdapterSetPacketFilterCapabilities(
        adapter: NETADAPTER,
        packet_filter: *const NET_ADAPTER_PACKET_FILTER_CAPABILITIES,
    ) -> () {
        NetAdapterSetPacketFilterCapabilitiesTableIndex
    }
);
