use core::ffi::c_void;

c_type!(
    pub struct NET_FRAGMENT_VIRTUAL_ADDRESS {
        pub virtual_address: *mut c_void,
    }
);

pub const NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_NAME: [u16; 27] =
    utf16!(b"ms_fragment_virtualaddress\0");
pub const NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_VERSION_1: u32 = 1;
