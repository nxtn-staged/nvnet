use core::{ffi::c_void, ptr};

// L814
c_type!(
    pub struct HANDLE(pub *mut c_void);
);

impl Default for HANDLE {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

// L1013
c_type!(
    #[must_use]
    pub struct NTSTATUS(pub i32);
);

// L1186
c_type!(
    pub struct LARGE_INTEGER(pub i64);
);

// L1497
c_type!(
    pub enum EVENT_TYPE {
        SynchronizationEvent = 1,
    }
);

// L1586
c_type!(
    pub struct UNICODE_STRING {
        pub Length: u16,
        pub MaximumLength: u16,
        pub Buffer: *mut u16,
    }
);

// L1650
c_type!(
    pub struct LIST_ENTRY {
        pub Flink: *mut LIST_ENTRY,
        pub Blink: *mut LIST_ENTRY,
    }
);

// L1841
c_type!(
    pub struct OBJECT_ATTRIBUTES;
);

// L1959
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($type:ty, $field:ident) => {
        unsafe {
            let uninit = core::mem::MaybeUninit::<$type>::uninit();
            let base_ptr = uninit.as_ptr();
            let field_ptr = core::ptr::addr_of!((*base_ptr).$field);
            field_ptr.cast::<u8>().offset_from(base_ptr.cast::<u8>()) as u16
        }
    };
}

#[macro_export]
macro_rules! RTL_FIELD_SIZE {
    ($type:ty, $field:ident) => {
        unsafe {
            let uninit = core::mem::MaybeUninit::<$type>::uninit();
            let base_ptr = uninit.as_ptr();
            let field_ptr = core::ptr::addr_of!((*base_ptr).$field);
            core::mem::size_of_val_raw(field_ptr) as u16
        }
    };
}

#[macro_export]
macro_rules! RTL_SIZEOF_THROUGH_FIELD {
    ($type:ty, $field:ident) => {
        $crate::FIELD_OFFSET!($type, $field) + $crate::RTL_FIELD_SIZE!($type, $field)
    };
}

// L2230
c_type!(
    pub struct KIRQL(pub u8);
);
