use crate::windows::wdf::kmdf::wdfglobals::WDF_DRIVER_GLOBALS;

extern "system" {
    pub static WdfDriverGlobals: *const WDF_DRIVER_GLOBALS;
}

c_type!(
    pub struct WDFFUNCENUM(isize);
);

impl WDFFUNCENUM {
    pub const WdfDeviceInitSetPnpPowerEventCallbacksTableIndex: isize = 55;
    pub const WdfDeviceInitSetFileObjectConfigTableIndex: isize = 71;
    pub const WdfDeviceCreateTableIndex: isize = 75;
    pub const WdfDeviceCreateSymbolicLinkTableIndex: isize = 80;
    pub const WdfDriverCreateTableIndex: isize = 116;
    pub const WdfFileObjectGetDeviceTableIndex: isize = 139;
    pub const WdfIoQueueCreateTableIndex: isize = 152;
    pub const WdfIoQueueGetDeviceTableIndex: isize = 157;
    pub const WdfObjectGetTypedContextWorkerTableIndex: isize = 202;
    pub const WdfRequestCompleteTableIndex: isize = 263;
    pub const WdfRequestRetrieveInputBufferTableIndex: isize = 269;
    pub const WdfRequestGetFileObjectTableIndex: isize = 277;
}
