use std::{mem, slice};

pub trait FromRawUtf16 {
    fn from_raw_utf16(s: *const u16, len: usize) -> Self;
}

impl FromRawUtf16 for String {
    fn from_raw_utf16(s: *const u16, len: usize) -> Self {
        unsafe { String::from_utf16(slice::from_raw_parts(s, len)).unwrap() }
    }
}

pub trait ToUtf16 {
    fn to_utf16(&self) -> Vec<u16>;
}

impl ToUtf16 for str {
    fn to_utf16(&self) -> Vec<u16> {
        let mut vec = self.encode_utf16().collect::<Vec<_>>();
        vec.push(0);
        vec
    }
}

pub trait AsBytesExt {
    fn as_bytes(&self) -> &[u8];

    fn as_mut_bytes(&mut self) -> &mut [u8];
}

impl<T> AsBytesExt for T {
    fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts((self as *const Self).cast(), mem::size_of::<Self>()) }
    }

    fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut((self as *mut Self).cast(), mem::size_of::<Self>()) }
    }
}
