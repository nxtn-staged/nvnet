use crate::windows::km::wdm::MDL;

c_type!(
    pub struct NET_FRAGMENT_MDL {
        pub mdl: *mut MDL,
    }
);

pub const NET_FRAGMENT_EXTENSION_MDL_NAME: [u16; 16] = utf16!(b"ms_fragment_mdl\0");
pub const NET_FRAGMENT_EXTENSION_MDL_VERSION_1: u32 = 1;
