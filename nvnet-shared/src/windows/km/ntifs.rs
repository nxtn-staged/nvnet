pub const FILE_DEVICE_NETWORK: u32 = 0x00000012;

pub const fn CTL_CODE(DeviceType: u32, Function: u32, Method: u32, Access: u32) -> u32 {
    (DeviceType << 16) | (Access << 14) | (Function << 2) | Method
}

pub const METHOD_BUFFERED: u32 = 0;

pub const FILE_ANY_ACCESS: u32 = 0;
