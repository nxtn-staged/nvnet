use std::{error::Error, fmt, mem::MaybeUninit, ptr};

use winapi::um::{
    errhandlingapi::GetLastError,
    winbase::{
        FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
    },
};

use crate::ext::FromRawUtf16;

pub struct WinError {
    error: u32,
}

impl WinError {
    pub fn new() -> Self {
        Self {
            error: unsafe { GetLastError() },
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = MaybeUninit::uninit();
        let num = unsafe {
            FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ALLOCATE_BUFFER,
                ptr::null(),
                self.error,
                0,
                buffer.as_mut_ptr() as _,
                0,
                ptr::null_mut(),
            )
        };
        if num == 0 {
            write!(f, "({})", self.error)
        } else {
            let buffer = unsafe { buffer.assume_init() };
            let message = String::from_raw_utf16(buffer, (num - 1) as _);
            unsafe { LocalFree(buffer as _) };
            write!(f, "{} ({})", message.trim_end(), self.error)
        }
    }
}

impl fmt::Debug for WinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Display for WinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

impl Error for WinError {}
