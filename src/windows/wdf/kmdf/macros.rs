macro_rules! wdf_fn {
    ($(#[$attr:meta])* $vis:vis fn $name:ident($($param_name:ident: $param_ty:ty),*$(,)?) -> $ret:ty{$index:ident}) => {
        $(#[$attr])*
        $vis unsafe fn $name($($param_name: $param_ty),*) -> $ret {
            crate::windows::wdf::kmdf::wdf::wdf_functions()
                .offset(crate::windows::wdf::kmdf::wdffuncenum::WDFFUNCENUM::$index)
                .cast::<extern "system" fn(driver_globals: *const crate::windows::wdf::kmdf::wdfglobals::WDF_DRIVER_GLOBALS, $($param_name: $param_ty),*) -> $ret>()
                .read()(crate::windows::wdf::kmdf::wdffuncenum::WdfDriverGlobals, $($param_name),*)
        }
    };
}
