use crate::windows::shared::ntdef::{HANDLE, LARGE_INTEGER, NTSTATUS};

// L2411
extern "system" {
    // #[irql_requires_max(APC_LEVEL)]
    pub fn RtlRandomEx(Seed: *mut u32) -> u32;
}

// L29547
extern "system" {
    // #[irql_requires_max(APC_LEVEL)]
    // ...
    pub fn ZwWaitForSingleObject(
        Handle: HANDLE,
        Alertable: bool,
        Timeout: *mut LARGE_INTEGER,
    ) -> NTSTATUS;
}
