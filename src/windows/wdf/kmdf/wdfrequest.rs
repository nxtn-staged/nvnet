use sal::*;

use crate::windows::{
    shared::ntdef::{NTSTATUS, PVOID},
    wdf::kmdf::wdftypes::{WDFFILEOBJECT, WDFREQUEST},
};

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfRequestComplete(request: WDFREQUEST, status: NTSTATUS) -> () {
        WdfRequestCompleteTableIndex
    }
);

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfRequestRetrieveInputBuffer(
        request: WDFREQUEST,
        minimum_required_length: usize,
        buffer: *mut PVOID,
        length: *mut usize,
    ) -> NTSTATUS {
        WdfRequestRetrieveInputBufferTableIndex
    }
);

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfRequestGetFileObject(request: WDFREQUEST) -> WDFFILEOBJECT {
        WdfRequestGetFileObjectTableIndex
    }
);
