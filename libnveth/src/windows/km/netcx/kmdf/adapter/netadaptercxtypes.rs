use core::mem;

use crate::windows::wdf::kmdf::wdftypes::WDFOBJECT;

win_handle!(NETADAPTER);
win_handle!(NETPACKETQUEUE);

impl From<NETADAPTER> for WDFOBJECT {
    fn from(handle: NETADAPTER) -> Self {
        unsafe { mem::transmute(handle) }
    }
}

impl From<NETPACKETQUEUE> for WDFOBJECT {
    fn from(handle: NETPACKETQUEUE) -> Self {
        unsafe { mem::transmute(handle) }
    }
}

c_type!(
    pub struct NETADAPTER_INIT;
);

c_type!(
    pub struct NET_DRIVER_GLOBALS;
);

type NETFUNC = extern "system" fn() -> ();

pub fn net_functions() -> *const NETFUNC {
    unsafe { &NetFunctions }
}

extern "system" {
    static NetFunctions: NETFUNC;
}
