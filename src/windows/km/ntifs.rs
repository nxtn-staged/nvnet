use sal::*;

use crate::windows::shared::ntdef::{HANDLE, LARGE_INTEGER, NTSTATUS};

extern "system" {
    #[irql_requires_max(APC_LEVEL)]
    pub fn RtlRandomEx(seed: *mut u32) -> u32;

    #[when(timeout.is_null(), irql_requires_max(APC_LEVEL))]
    pub fn ZwWaitForSingleObject(
        handle: HANDLE,
        alertable: bool,
        timeout: *const LARGE_INTEGER,
    ) -> NTSTATUS;
}
