use crate::windows::shared::netcx::shared::net::{
    extension::{NetExtensionGetData, NET_EXTENSION},
    virtualaddresstypes::NET_FRAGMENT_VIRTUAL_ADDRESS,
};

pub unsafe fn NetExtensionGetFragmentVirtualAddress(
    extension: *const NET_EXTENSION,
    index: u32,
) -> *mut NET_FRAGMENT_VIRTUAL_ADDRESS {
    NetExtensionGetData(extension, index).cast()
}
