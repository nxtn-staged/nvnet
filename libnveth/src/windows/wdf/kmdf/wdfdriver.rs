use core::{default::default, mem};

use libnveth_macros::*;

use crate::windows::{
    km::wdm::DRIVER_OBJECT,
    shared::ntdef::{NTSTATUS, UNICODE_STRING},
    wdf::kmdf::{
        wdfobject::WDF_OBJECT_ATTRIBUTES,
        wdftypes::{WDFDEVICE_INIT, WDFDRIVER},
    },
};

c_type!(
    pub type PFN_WDF_DRIVER_DEVICE_ADD =
        fn(driver: WDFDRIVER, device_init: *mut WDFDEVICE_INIT) -> NTSTATUS;
);

c_type!(
    pub type PFN_WDF_DRIVER_UNLOAD = fn(driver: WDFDRIVER) -> ();
);

c_type!(
    pub struct WDF_DRIVER_CONFIG {
        pub size: u32,
        pub evt_driver_device_add: PFN_WDF_DRIVER_DEVICE_ADD,
        pub evt_driver_unload: PFN_WDF_DRIVER_UNLOAD,
        pub driver_init_flags: u32,
        pub driver_pool_tag: u32,
    }
);

pub fn WDF_DRIVER_CONFIG_INIT(
    evt_driver_device_add: PFN_WDF_DRIVER_DEVICE_ADD,
) -> WDF_DRIVER_CONFIG {
    WDF_DRIVER_CONFIG {
        size: mem::size_of::<WDF_DRIVER_CONFIG>() as _,
        evt_driver_device_add,
        ..default()
    }
}

wdf_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn WdfDriverCreate(
        driver_object: *const DRIVER_OBJECT,
        registry_path: *const UNICODE_STRING,
        driver_attributes: *const WDF_OBJECT_ATTRIBUTES,
        driver_config: *const WDF_DRIVER_CONFIG,
        driver: *mut WDFDRIVER,
    ) -> NTSTATUS {
        WdfDriverCreateTableIndex
    }
);
