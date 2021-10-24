use crate::bindings::Windows::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::{
        CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_EXISTING,
    },
    System::SystemServices::DeviceIoControl,
};
use std::{default::default, mem, ptr};
use windows::{Handle, Result};

pub struct Device(HANDLE);

impl Device {
    pub fn open(path: impl AsRef<str>) -> Result<Self> {
        let handle = unsafe {
            CreateFileW(
                path.as_ref(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_NONE,
                ptr::null(),
                OPEN_EXISTING,
                default(),
                HANDLE::default(),
            )
            .ok()?
        };
        Ok(Self(handle))
    }

    pub fn control_in<T>(&self, control_code: u32, val: T) -> Result<()> {
        self.control_in_ref(control_code, &val)?;
        Ok(())
    }

    pub fn control_in_ref<T: ?Sized>(&self, control_code: u32, val: &T) -> Result<()> {
        unsafe {
            DeviceIoControl(
                self.0,
                control_code,
                val as *const _ as *const _,
                mem::size_of_val(val) as u32,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            )
            .ok()?
        };
        Ok(())
    }
}
