pub const FILE_DEVICE_NETWORK: u32 = 0x00000012;

pub const fn CTL_CODE(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

pub const METHOD_BUFFERED: u32 = 0;

pub const FILE_ANY_ACCESS: u32 = 0;

pub const IO_NO_INCREMENT: i8 = 0;
