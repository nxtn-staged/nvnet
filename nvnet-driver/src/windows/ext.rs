use core::mem;

use crate::windows::{
    shared::{
        ndis::types::NDIS_STATUS,
        ntdef::{NTSTATUS, UNICODE_STRING},
    },
    Result,
};

pub trait OkExt {
    fn ok(self) -> Result<()>;
}

impl OkExt for NDIS_STATUS {
    fn ok(self) -> Result<()> {
        if self.0 >= 0 {
            Ok(())
        } else {
            Err(self.into())
        }
    }
}

impl OkExt for NTSTATUS {
    fn ok(self) -> Result<()> {
        if self.0 >= 0 {
            Ok(())
        } else {
            Err(self)
        }
    }
}

pub trait UnicodeStringExt {
    unsafe fn new(chars: &mut [u16]) -> Self;
}

impl UnicodeStringExt for UNICODE_STRING {
    unsafe fn new(chars: &mut [u16]) -> Self {
        let size = mem::size_of_val(chars) as u16;
        Self {
            Length: size,
            MaximumLength: size,
            Buffer: chars.as_mut_ptr(),
        }
    }
}
