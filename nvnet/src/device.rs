use std::{ffi::c_void, mem, ptr};

use winapi::{
    shared::minwindef::FALSE,
    um::{
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        ioapiset::DeviceIoControl,
        winnt::{FILE_GENERIC_READ, FILE_GENERIC_WRITE},
    },
};

use crate::{error::WinError, ext::ToUtf16};

pub struct Device(*mut c_void);

impl Device {
    pub fn open(path: impl AsRef<str>) -> Result<Self, WinError> {
        let path = path.as_ref().to_utf16();
        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                0,
                ptr::null_mut(),
                OPEN_EXISTING,
                0,
                ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            Err(WinError::new())
        } else {
            Ok(Self(handle))
        }
    }

    pub fn control_in<T>(&self, control: u32, value: T) -> Result<(), WinError> {
        self.control_in_ref(control, &value)
    }

    pub fn control_in_ref<T>(&self, control: u32, value: &T) -> Result<(), WinError> {
        let success = unsafe {
            DeviceIoControl(
                self.0,
                control,
                value as *const _ as *mut _,
                mem::size_of::<T>() as _,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if success == FALSE {
            Err(WinError::new())
        } else {
            Ok(())
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}
