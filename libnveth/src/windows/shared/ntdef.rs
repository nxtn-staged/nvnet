use core::{ffi::c_void, ptr};

pub type PVOID = *mut c_void;

c_type!(
    #[derive(Debug)]
    pub struct HANDLE(*mut c_void);
);

impl Default for HANDLE {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

c_type!(
    #[derive(Debug, PartialEq, Eq)]
    pub struct NTSTATUS(pub i32);
);

pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status.0 >= 0
}

c_type!(
    pub struct LARGE_INTEGER;
);

pub type PHYSICAL_ADDRESS = LARGE_INTEGER;

c_type!(
    pub enum EVENT_TYPE {
        NotificationEvent = 0,
        SynchronizationEvent = 1,
    }
);

c_type!(
    pub struct UNICODE_STRING {
        pub length: u16,
        pub maximum_length: u16,
        pub buffer: *mut u16,
    }
);

c_type!(
    pub struct LIST_ENTRY {
        pub flink: *mut LIST_ENTRY,
        pub blink: *mut LIST_ENTRY,
    }
);

c_type!(
    pub struct SINGLE_LIST_ENTRY {
        pub next: *mut SINGLE_LIST_ENTRY,
    }
);

c_type!(
    pub struct OBJECT_ATTRIBUTES;
);

c_type!(
    pub struct KIRQL(pub u8);
);
