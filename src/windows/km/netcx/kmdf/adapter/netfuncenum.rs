use crate::windows::km::netcx::kmdf::adapter::netadaptercxtypes::NET_DRIVER_GLOBALS;

extern "system" {
    pub static NetDriverGlobals: *const NET_DRIVER_GLOBALS;
}

c_type!(
    pub struct NETFUNCENUM(isize);
);

impl NETFUNCENUM {
    pub const NetAdapterInitAllocateTableIndex: isize = 0;
    pub const NetAdapterInitFreeTableIndex: isize = 1;
    pub const NetAdapterInitSetDatapathCallbacksTableIndex: isize = 2;
    pub const NetAdapterCreateTableIndex: isize = 3;
    pub const NetAdapterStartTableIndex: isize = 4;
    pub const NetAdapterSetLinkLayerCapabilitiesTableIndex: isize = 6;
    pub const NetAdapterSetLinkLayerMtuSizeTableIndex: isize = 7;
    pub const NetAdapterSetDataPathCapabilitiesTableIndex: isize = 14;
    pub const NetAdapterSetLinkStateTableIndex: isize = 15;
    pub const NetAdapterSetPermanentLinkLayerAddressTableIndex: isize = 18;
    pub const NetAdapterSetCurrentLinkLayerAddressTableIndex: isize = 19;
    pub const NetDeviceInitConfigTableIndex: isize = 46;
    pub const NetRxQueueCreateTableIndex: isize = 58;
    pub const NetRxQueueNotifyMoreReceivedPacketsAvailableTableIndex: isize = 59;
    pub const NetRxQueueGetRingCollectionTableIndex: isize = 61;
    pub const NetRxQueueGetExtensionTableIndex: isize = 62;
    pub const NetTxQueueCreateTableIndex: isize = 63;
    pub const NetTxQueueNotifyMoreCompletedPacketsAvailableTableIndex: isize = 64;
    pub const NetTxQueueGetRingCollectionTableIndex: isize = 66;
    pub const NetTxQueueGetExtensionTableIndex: isize = 67;
    pub const NetAdapterSetPacketFilterCapabilitiesTableIndex: isize = 82;
}
