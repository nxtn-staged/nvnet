use libnveth_macros::*;

use crate::windows::{shared::ntdef::NTSTATUS, wdf::kmdf::wdftypes::WDFDEVICE_INIT};

net_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn NetDeviceInitConfig(device_init: *mut WDFDEVICE_INIT) -> NTSTATUS {
        NetDeviceInitConfigTableIndex
    }
);
