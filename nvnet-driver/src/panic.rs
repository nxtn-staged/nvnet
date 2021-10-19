use core::panic::PanicInfo;

use crate::windows::km::wdm::{__fastfail, FAST_FAIL_FATAL_APP_EXIT};

#[panic_handler]
fn rust_begin_unwind(_info: &PanicInfo) -> ! {
    unsafe { __fastfail(FAST_FAIL_FATAL_APP_EXIT) };
}
