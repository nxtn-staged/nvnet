use core::{default::default, mem};

use sal::*;

use crate::windows::{
    shared::ntdef::{NTSTATUS, UNICODE_STRING},
    wdf::kmdf::{
        wdfobject::WDF_OBJECT_ATTRIBUTES,
        wdftypes::{
            WDFCMRESLIST, WDFDEVICE, WDFDEVICE_INIT, WDFFILEOBJECT, WDFREQUEST, WDF_TRI_STATE,
        },
    },
};

c_type!(
    pub enum WDF_FILEOBJECT_CLASS {
        WdfFileObjectInvalid = 0,
        WdfFileObjectWdfCannotUseFsContexts = 4,
    }
);

c_type!(
    pub type PFN_WDF_DEVICE_FILE_CREATE =
        fn(device: WDFDEVICE, request: WDFREQUEST, file_object: WDFFILEOBJECT) -> ();
);

c_type!(
    pub type PFN_WDF_FILE_CLOSE = fn(file_object: WDFFILEOBJECT) -> ();
);

c_type!(
    pub type PFN_WDF_FILE_CLEANUP = fn() -> !;
);

c_type!(
    pub struct WDF_FILEOBJECT_CONFIG {
        pub size: u32,
        pub evt_device_file_create: PFN_WDF_DEVICE_FILE_CREATE,
        pub evt_file_close: PFN_WDF_FILE_CLOSE,
        pub evt_file_cleanup: PFN_WDF_FILE_CLEANUP,
        pub auto_forward_cleanup_close: WDF_TRI_STATE,
        pub file_object_class: WDF_FILEOBJECT_CLASS,
    }
);

pub fn WDF_FILEOBJECT_CONFIG_INIT(
    evt_device_file_create: PFN_WDF_DEVICE_FILE_CREATE,
    evt_file_close: PFN_WDF_FILE_CLOSE,
    evt_file_cleanup: PFN_WDF_FILE_CLEANUP,
) -> WDF_FILEOBJECT_CONFIG {
    WDF_FILEOBJECT_CONFIG {
        size: mem::size_of::<WDF_FILEOBJECT_CONFIG>() as _,
        evt_device_file_create,
        evt_file_close,
        evt_file_cleanup,
        file_object_class: WDF_FILEOBJECT_CLASS::WdfFileObjectWdfCannotUseFsContexts,
        auto_forward_cleanup_close: WDF_TRI_STATE::WdfUseDefault,
        ..default()
    }
}

c_type!(
    pub type PFN_WDF_DEVICE_D0_ENTRY = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_D0_ENTRY_POST_INTERRUPTS_ENABLED = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_D0_EXIT = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_D0_EXIT_PRE_INTERRUPTS_DISABLED = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_PREPARE_HARDWARE = fn(
        device: WDFDEVICE,
        resources_raw: WDFCMRESLIST,
        resources_translated: WDFCMRESLIST,
    ) -> NTSTATUS;
);

c_type!(
    pub type PFN_WDF_DEVICE_RELEASE_HARDWARE =
        fn(device: WDFDEVICE, resources_translated: WDFCMRESLIST) -> NTSTATUS;
);

c_type!(
    pub type PFN_WDF_DEVICE_SELF_MANAGED_IO_CLEANUP = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_SELF_MANAGED_IO_FLUSH = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_SELF_MANAGED_IO_INIT = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_SELF_MANAGED_IO_SUSPEND = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_SELF_MANAGED_IO_RESTART = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_QUERY_STOP = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_QUERY_REMOVE = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_SURPRISE_REMOVAL = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_USAGE_NOTIFICATION = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_USAGE_NOTIFICATION_EX = fn() -> !;
);

c_type!(
    pub type PFN_WDF_DEVICE_RELATIONS_QUERY = fn() -> !;
);

c_type!(
    pub struct WDF_PNPPOWER_EVENT_CALLBACKS {
        pub size: u32,
        pub evt_device_d0_entry: PFN_WDF_DEVICE_D0_ENTRY,
        pub evt_device_d0_entry_post_interrupts_enabled:
            PFN_WDF_DEVICE_D0_ENTRY_POST_INTERRUPTS_ENABLED,
        pub evt_device_d0_exit: PFN_WDF_DEVICE_D0_EXIT,
        pub evt_device_d0_exit_pre_interrupts_disabled:
            PFN_WDF_DEVICE_D0_EXIT_PRE_INTERRUPTS_DISABLED,
        pub evt_device_prepare_hardware: PFN_WDF_DEVICE_PREPARE_HARDWARE,
        pub evt_device_release_hardware: PFN_WDF_DEVICE_RELEASE_HARDWARE,
        pub evt_device_self_managed_io_cleanup: PFN_WDF_DEVICE_SELF_MANAGED_IO_CLEANUP,
        pub evt_device_self_managed_io_flush: PFN_WDF_DEVICE_SELF_MANAGED_IO_FLUSH,
        pub evt_device_self_managed_io_init: PFN_WDF_DEVICE_SELF_MANAGED_IO_INIT,
        pub evt_device_self_managed_io_suspend: PFN_WDF_DEVICE_SELF_MANAGED_IO_SUSPEND,
        pub evt_device_self_managed_io_restart: PFN_WDF_DEVICE_SELF_MANAGED_IO_RESTART,
        pub evt_device_surprise_removal: PFN_WDF_DEVICE_SURPRISE_REMOVAL,
        pub evt_device_query_remove: PFN_WDF_DEVICE_QUERY_REMOVE,
        pub evt_device_query_stop: PFN_WDF_DEVICE_QUERY_STOP,
        pub evt_device_usage_notification: PFN_WDF_DEVICE_USAGE_NOTIFICATION,
        pub evt_device_relations_query: PFN_WDF_DEVICE_RELATIONS_QUERY,
        pub evt_device_usage_notification_ex: PFN_WDF_DEVICE_USAGE_NOTIFICATION_EX,
    }
);

pub fn WDF_PNPPOWER_EVENT_CALLBACKS_INIT() -> WDF_PNPPOWER_EVENT_CALLBACKS {
    WDF_PNPPOWER_EVENT_CALLBACKS {
        size: mem::size_of::<WDF_PNPPOWER_EVENT_CALLBACKS>() as _,
        ..default()
    }
}

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfDeviceInitSetPnpPowerEventCallbacks(
        device_init: *mut WDFDEVICE_INIT,
        pnp_power_event_callbacks: *const WDF_PNPPOWER_EVENT_CALLBACKS,
    ) -> () {
        WdfDeviceInitSetPnpPowerEventCallbacksTableIndex
    }
);

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfDeviceInitSetFileObjectConfig(
        device_init: *mut WDFDEVICE_INIT,
        file_object_config: *const WDF_FILEOBJECT_CONFIG,
        file_object_attributes: *const WDF_OBJECT_ATTRIBUTES,
    ) -> () {
        WdfDeviceInitSetFileObjectConfigTableIndex
    }
);

wdf_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn WdfDeviceCreate(
        device_init: *mut *mut WDFDEVICE_INIT,
        device_attributes: *const WDF_OBJECT_ATTRIBUTES,
        device: *mut WDFDEVICE,
    ) -> NTSTATUS {
        WdfDeviceCreateTableIndex
    }
);

wdf_fn!(
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn WdfDeviceCreateSymbolicLink(
        device: WDFDEVICE,
        symbolic_link_name: *const UNICODE_STRING,
    ) -> NTSTATUS {
        WdfDeviceCreateSymbolicLinkTableIndex
    }
);
