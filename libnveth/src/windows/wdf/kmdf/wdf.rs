type WDFFUNC = extern "system" fn() -> ();

pub fn wdf_functions() -> *const WDFFUNC {
    unsafe { WdfFunctions_01015 }
}

extern "system" {
    static WdfFunctions_01015: *const WDFFUNC;
}
