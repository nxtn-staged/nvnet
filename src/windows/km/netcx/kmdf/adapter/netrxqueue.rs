use sal::*;

use crate::windows::{
    km::netcx::kmdf::adapter::{
        netadaptercxtypes::NETPACKETQUEUE, netadapterpacket::NET_EXTENSION_QUERY,
        netpacketqueue::NET_PACKET_QUEUE_CONFIG,
    },
    shared::{
        netcx::shared::net::{extension::NET_EXTENSION, ringcollection::NET_RING_COLLECTION},
        ntdef::NTSTATUS,
    },
    wdf::kmdf::wdfobject::WDF_OBJECT_ATTRIBUTES,
};

c_type!(
    pub struct NETRXQUEUE_INIT;
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetRxQueueCreate(
        net_rx_queue_init: *mut NETRXQUEUE_INIT,
        rx_queue_attributes: *const WDF_OBJECT_ATTRIBUTES,
        configuration: *const NET_PACKET_QUEUE_CONFIG,
        packet_queue: *mut NETPACKETQUEUE,
    ) -> NTSTATUS {
        NetRxQueueCreateTableIndex
    }
);

net_fn!(
    #[irql_requires_max(HIGH_LEVEL)]
    pub fn NetRxQueueNotifyMoreReceivedPacketsAvailable(packet_queue: NETPACKETQUEUE) -> () {
        NetRxQueueNotifyMoreReceivedPacketsAvailableTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetRxQueueGetRingCollection(packet_queue: NETPACKETQUEUE) -> *const NET_RING_COLLECTION {
        NetRxQueueGetRingCollectionTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetRxQueueGetExtension(
        packet_queue: NETPACKETQUEUE,
        query: *const NET_EXTENSION_QUERY,
        extension: *mut NET_EXTENSION,
    ) -> () {
        NetRxQueueGetExtensionTableIndex
    }
);
