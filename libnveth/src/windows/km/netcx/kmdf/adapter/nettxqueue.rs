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
    pub struct NETTXQUEUE_INIT;
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetTxQueueCreate(
        net_tx_queue_init: *mut NETTXQUEUE_INIT,
        tx_queue_attributes: *const WDF_OBJECT_ATTRIBUTES,
        configuration: *const NET_PACKET_QUEUE_CONFIG,
        packet_queue: *mut NETPACKETQUEUE,
    ) -> NTSTATUS {
        NetTxQueueCreateTableIndex
    }
);

net_fn!(
    #[irql_requires_max(HIGH_LEVEL)]
    pub fn NetTxQueueNotifyMoreCompletedPacketsAvailable(packet_queue: NETPACKETQUEUE) -> () {
        NetTxQueueNotifyMoreCompletedPacketsAvailableTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetTxQueueGetRingCollection(packet_queue: NETPACKETQUEUE) -> *const NET_RING_COLLECTION {
        NetTxQueueGetRingCollectionTableIndex
    }
);

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetTxQueueGetExtension(
        packet_queue: NETPACKETQUEUE,
        query: *const NET_EXTENSION_QUERY,
        extension: *mut NET_EXTENSION,
    ) -> () {
        NetTxQueueGetExtensionTableIndex
    }
);
