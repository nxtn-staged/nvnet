use core::mem;

use crate::windows::shared::netcx::shared::net::extension::NET_EXTENSION_TYPE;

c_type!(
    pub struct NET_EXTENSION_QUERY {
        size: u32,
        name: *const u16,
        version: u32,
        r#type: NET_EXTENSION_TYPE,
    }
);

pub fn NET_EXTENSION_QUERY_INIT(
    name: *const u16,
    version: u32,
    r#type: NET_EXTENSION_TYPE,
) -> NET_EXTENSION_QUERY {
    NET_EXTENSION_QUERY {
        size: mem::size_of::<NET_EXTENSION_QUERY>() as _,
        name,
        version,
        r#type,
    }
}
