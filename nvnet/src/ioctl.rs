use crate::windows::km::ntddk::{CTL_CODE, FILE_ANY_ACCESS, FILE_DEVICE_NETWORK, METHOD_BUFFERED};

const fn veth_ctl_code(function_index: u32) -> u32 {
    CTL_CODE(
        FILE_DEVICE_NETWORK,
        function_index + 0x800,
        METHOD_BUFFERED,
        FILE_ANY_ACCESS,
    )
}

pub const IOCTL_VETH_SET_CONNECT_STATE: u32 = veth_ctl_code(0);
pub const IOCTL_VETH_SET_LOCAL_ADDR: u32 = veth_ctl_code(1);
pub const IOCTL_VETH_ADD_REMOTE_PEER: u32 = veth_ctl_code(2);
