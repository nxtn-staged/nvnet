use core::ffi::c_void;

c_type!(
    pub enum NET_EXTENSION_TYPE {
        NetExtensionTypePacket = 1,
        NetExtensionTypeFragment,
    }
);

c_type!(
    pub struct NET_EXTENSION {
        reserved: [*mut c_void; 4],
        u: NET_EXTENSION_u,
    }
);

c_type!(
    union NET_EXTENSION_u {
        enabled: bool,
        reserved1: *mut c_void,
    }
);

pub unsafe fn NetExtensionGetData(extension: *const NET_EXTENSION, index: u32) -> *mut c_void {
    (*extension).reserved[0]
        .cast::<u8>()
        .offset((*extension).reserved[1] as isize * index as isize)
        .cast()
}
