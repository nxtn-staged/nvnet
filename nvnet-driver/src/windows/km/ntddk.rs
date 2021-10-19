use core::ffi::c_void;

use crate::windows::km::wdm::ExFreePoolWithTag;

// L9900
pub unsafe fn ExFreePool(P: *mut c_void) {
    ExFreePoolWithTag(P, 0);
}
