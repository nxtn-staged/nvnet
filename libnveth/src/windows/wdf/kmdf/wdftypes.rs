use core::ffi::c_void;

c_type!(
    pub enum WDF_TRI_STATE {
        WdfFalse = 0,
        WdfTrue = 1,
        WdfUseDefault = 2,
    }
);

c_type!(
    pub struct WDFDEVICE_INIT;
);

c_type!(
    pub struct WDFOBJECT(pub *mut c_void);
);

declare_handle!(WDFDRIVER);
declare_handle!(WDFDEVICE);
declare_handle!(WDFQUEUE);
declare_handle!(WDFREQUEST);
declare_handle!(WDFFILEOBJECT);
declare_handle!(WDFDMAENABLER);
declare_handle!(WDFCMRESLIST);

impl From<WDFDEVICE> for WDFOBJECT {
    fn from(handle: WDFDEVICE) -> Self {
        Self(handle.0)
    }
}

impl From<WDFFILEOBJECT> for WDFOBJECT {
    fn from(handle: WDFFILEOBJECT) -> Self {
        Self(handle.0)
    }
}
