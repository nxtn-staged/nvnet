use crate::windows::wdf::kmdf::wdftypes::WDFOBJECT;

declare_handle!(NETADAPTER);
declare_handle!(NETPACKETQUEUE);

impl From<NETADAPTER> for WDFOBJECT {
    fn from(handle: NETADAPTER) -> Self {
        Self(handle.0)
    }
}

impl From<NETPACKETQUEUE> for WDFOBJECT {
    fn from(handle: NETPACKETQUEUE) -> Self {
        Self(handle.0)
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
