c_type!(
    pub struct NET_FRAGMENT_VIRTUAL_ADDRESS {
        pub virtual_address: *mut u8,
    }
);

pub const NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_NAME: [u16; 27] =
    utf16!(b"ms_fragment_virtualaddress\0");
pub const NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_VERSION_1: u32 = 1;
