use crate::windows::shared::netcx::shared::net::{
    extension::{NetExtensionGetData, NET_EXTENSION},
    mdltypes::NET_FRAGMENT_MDL,
};

pub unsafe fn NetExtensionGetFragmentMdl(
    extension: *const NET_EXTENSION,
    index: u32,
) -> *mut NET_FRAGMENT_MDL {
    NetExtensionGetData(extension, index).cast()
}
