use core::{default::default, mem};

use crate::windows::{shared::ntdef::PVOID, wdf::kmdf::wdftypes::WDFOBJECT};

c_type!(
    pub enum WDF_EXECUTION_LEVEL {
        WdfExecutionLevelInvalid = 0x00,
        WdfExecutionLevelInheritFromParent,
    }
);

c_type!(
    pub enum WDF_SYNCHRONIZATION_SCOPE {
        WdfSynchronizationScopeInvalid = 0x00,
        WdfSynchronizationScopeInheritFromParent,
    }
);

c_type!(
    pub type PFN_WDF_OBJECT_CONTEXT_CLEANUP = fn(object: WDFOBJECT) -> ();
);

c_type!(
    pub type PFN_WDF_OBJECT_CONTEXT_DESTROY = fn(object: WDFOBJECT) -> ();
);

c_type!(
    pub struct WDF_OBJECT_ATTRIBUTES {
        pub size: u32,
        pub evt_cleanup_callback: PFN_WDF_OBJECT_CONTEXT_CLEANUP,
        pub evt_destroy_callback: PFN_WDF_OBJECT_CONTEXT_DESTROY,
        pub execution_level: WDF_EXECUTION_LEVEL,
        pub synchronization_scope: WDF_SYNCHRONIZATION_SCOPE,
        pub parent_object: WDFOBJECT,
        pub context_size_override: usize,
        pub context_type_info: *const WDF_OBJECT_CONTEXT_TYPE_INFO,
    }
);

pub fn WDF_OBJECT_ATTRIBUTES_INIT() -> WDF_OBJECT_ATTRIBUTES {
    WDF_OBJECT_ATTRIBUTES {
        size: mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as _,
        execution_level: WDF_EXECUTION_LEVEL::WdfExecutionLevelInheritFromParent,
        synchronization_scope: WDF_SYNCHRONIZATION_SCOPE::WdfSynchronizationScopeInheritFromParent,
        ..default()
    }
}

c_type!(
    pub type PFN_GET_UNIQUE_CONTEXT_TYPE = fn() -> !;
);

c_type!(
    pub struct WDF_OBJECT_CONTEXT_TYPE_INFO {
        pub size: u32,
        pub context_name: *const u8,
        pub context_size: usize,
        pub unique_type: *const WDF_OBJECT_CONTEXT_TYPE_INFO,
        pub evt_driver_get_unique_context_type: PFN_GET_UNIQUE_CONTEXT_TYPE,
    }
);

wdf_fn!(
    pub fn WdfObjectGetTypedContextWorker(
        handle: WDFOBJECT,
        type_info: *const WDF_OBJECT_CONTEXT_TYPE_INFO,
    ) -> PVOID {
        WdfObjectGetTypedContextWorkerTableIndex
    }
);
