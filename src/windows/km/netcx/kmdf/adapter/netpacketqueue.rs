use core::{default::default, mem};

use crate::windows::km::netcx::kmdf::adapter::netadaptercxtypes::NETPACKETQUEUE;

c_type!(
    pub type PFN_PACKET_QUEUE_START = fn(packet_queue: NETPACKETQUEUE) -> ();
);

c_type!(
    pub type PFN_PACKET_QUEUE_STOP = fn(packet_queue: NETPACKETQUEUE) -> ();
);

c_type!(
    pub type PFN_PACKET_QUEUE_CANCEL = fn(packet_queue: NETPACKETQUEUE) -> ();
);

c_type!(
    pub type PFN_PACKET_QUEUE_SET_NOTIFICATION_ENABLED =
        fn(packet_queue: NETPACKETQUEUE, notification_enabled: bool) -> ();
);

c_type!(
    pub type PFN_PACKET_QUEUE_ADVANCE = fn(packet_queue: NETPACKETQUEUE) -> ();
);

c_type!(
    pub struct NET_PACKET_QUEUE_CONFIG {
        pub size: u32,
        pub evt_start: PFN_PACKET_QUEUE_START,
        pub evt_stop: PFN_PACKET_QUEUE_STOP,
        pub evt_advance: PFN_PACKET_QUEUE_ADVANCE,
        pub evt_set_notification_enabled: PFN_PACKET_QUEUE_SET_NOTIFICATION_ENABLED,
        pub evt_cancel: PFN_PACKET_QUEUE_CANCEL,
    }
);

pub fn NET_PACKET_QUEUE_CONFIG_INIT(
    evt_advance: PFN_PACKET_QUEUE_ADVANCE,
    evt_set_notification_enabled: PFN_PACKET_QUEUE_SET_NOTIFICATION_ENABLED,
    evt_cancel: PFN_PACKET_QUEUE_CANCEL,
) -> NET_PACKET_QUEUE_CONFIG {
    NET_PACKET_QUEUE_CONFIG {
        size: mem::size_of::<NET_PACKET_QUEUE_CONFIG>() as _,
        evt_advance,
        evt_set_notification_enabled,
        evt_cancel,
        ..default()
    }
}
