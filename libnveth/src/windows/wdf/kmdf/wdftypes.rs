use core::mem;

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

win_handle!(WDFOBJECT);
win_handle!(WDFDRIVER);
win_handle!(WDFDEVICE);
win_handle!(WDFQUEUE);
win_handle!(WDFREQUEST);
win_handle!(WDFFILEOBJECT);
win_handle!(WDFDMAENABLER);
win_handle!(WDFCMRESLIST);

impl From<WDFDEVICE> for WDFOBJECT {
    fn from(handle: WDFDEVICE) -> Self {
        unsafe { mem::transmute(handle) }
    }
}

impl From<WDFFILEOBJECT> for WDFOBJECT {
    fn from(handle: WDFFILEOBJECT) -> Self {
        unsafe { mem::transmute(handle) }
    }
}
