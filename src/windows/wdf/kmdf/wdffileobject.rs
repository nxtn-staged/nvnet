use sal::*;

use crate::windows::wdf::kmdf::wdftypes::{WDFDEVICE, WDFFILEOBJECT};

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfFileObjectGetDevice(file_object: WDFFILEOBJECT) -> WDFDEVICE {
        WdfFileObjectGetDeviceTableIndex
    }
);
