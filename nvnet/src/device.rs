use std::{default::default, mem, ops::BitOr, ptr};

use winapi::um::handleapi::INVALID_HANDLE_VALUE;

use crate::{
    bindings::windows::win32::{
        file_system::{CreateFileW, FILE_ACCESS_FLAGS, FILE_CREATE_FLAGS, FILE_SHARE_FLAGS},
        system_services::{DeviceIoControl, HANDLE},
        windows_programming::CloseHandle,
    },
    error::WinError,
    ext::ToUtf16,
};

impl BitOr for FILE_ACCESS_FLAGS {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self.0 | rhs.0).into()
    }
}

pub struct Device(HANDLE);

impl Device {
    pub fn open(path: impl AsRef<str>) -> Result<Self, WinError> {
        let path = path.as_ref().to_utf16();
        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                FILE_ACCESS_FLAGS::FILE_GENERIC_READ | FILE_ACCESS_FLAGS::FILE_GENERIC_WRITE,
                FILE_SHARE_FLAGS::FILE_SHARE_NONE,
                ptr::null_mut(),
                FILE_CREATE_FLAGS::OPEN_EXISTING,
                default(),
                default(),
            )
        };
        if handle.0 == INVALID_HANDLE_VALUE as isize {
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
        if !success.as_bool() {
            Err(WinError::new())
        } else {
            Ok(())
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let success = unsafe { CloseHandle(self.0) };
        debug_assert_eq!(success.as_bool(), true);
    }
}
