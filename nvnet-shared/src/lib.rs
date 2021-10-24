#![no_std]

pub mod crypto;
mod windows;

use crate::windows::km::ntifs::{CTL_CODE, FILE_ANY_ACCESS, FILE_DEVICE_NETWORK, METHOD_BUFFERED};

const fn vnet_ctl_code(function_index: u32) -> u32 {
    CTL_CODE(
        FILE_DEVICE_NETWORK,
        function_index + 0x800,
        METHOD_BUFFERED,
        FILE_ANY_ACCESS,
    )
}

pub const IOCTL_VNET_SET_CONNECT_STATE: u32 = vnet_ctl_code(0);
pub const IOCTL_VNET_SET_LOCAL_ENDPOINT: u32 = vnet_ctl_code(1);
pub const IOCTL_VNET_SET_REMOTE_ENDPOINT: u32 = vnet_ctl_code(2);
pub const IOCTL_VNET_SET_REMOTE_SECRET_KEY: u32 = vnet_ctl_code(3);
