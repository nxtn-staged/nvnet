use core::{ffi::c_void, ptr};

use crate::windows::shared::ntdef::NTSTATUS;

// L11
c_type!(
    pub struct NDIS_HANDLE(pub *mut c_void);
);

impl Default for NDIS_HANDLE {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

// L13
c_type!(
    #[must_use]
    pub struct NDIS_STATUS(pub i32);
);

impl From<NTSTATUS> for NDIS_STATUS {
    fn from(status: NTSTATUS) -> Self {
        Self(status.0)
    }
}

impl From<NDIS_STATUS> for NTSTATUS {
    fn from(status: NDIS_STATUS) -> Self {
        Self(status.0)
    }
}
