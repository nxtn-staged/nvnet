use crate::windows::Result;

macro_rules! trace_println {
    ($format:expr, $($arg:expr),*) => {
        unsafe {
            crate::windows::km::wdm::DbgPrintEx(
                crate::windows::shared::dpfilter::DPFLTR_TYPE::DPFLTR_IHVNETWORK_ID,
                crate::windows::shared::dpfilter::DPFLTR_TRACE_LEVEL,
                concat!($format, "\n\0").as_ptr(),
                $($arg),*
            )
        };
    };
}

pub trait ResultExt {
    fn context_exit(self, name: &'static str) -> Self;
}

impl ResultExt for Result<()> {
    fn context_exit(self, name: &'static str) -> Self {
        if let Err(status) = self {
            trace_println!("%.*s - 0x%08x", name.len() as i32, name.as_ptr(), status);
        }
        self
    }
}
