macro_rules! net_fn {
    ($(#[$attr:meta])* $vis:vis fn $name:ident($($param_name:ident: $param_ty:ty),*$(,)?) -> $ret:ty{$index:ident}) => {
        $(#[$attr])*
        $vis unsafe fn $name($($param_name: $param_ty),*) -> $ret {
            crate::windows::km::netcx::kmdf::adapter::netadaptercxtypes::net_functions()
                .offset(crate::windows::km::netcx::kmdf::adapter::netfuncenum::NETFUNCENUM::$index)
                .cast::<extern "system" fn(driver_globals: *const crate::windows::km::netcx::kmdf::adapter::netadaptercxtypes::NET_DRIVER_GLOBALS, $($param_name: $param_ty),*) -> $ret>()
                .read()(crate::windows::km::netcx::kmdf::adapter::netfuncenum::NetDriverGlobals, $($param_name),*)
        }
    };
}
