pub const ETH_LENGTH_OF_ADDRESS: u16 = 6;

pub fn ETH_IS_MULTICAST(address: &[u8; ETH_LENGTH_OF_ADDRESS as _]) -> bool {
    address[0] & 0x01 != 0
}

pub fn ETH_IS_BROADCAST(address: &[u8; ETH_LENGTH_OF_ADDRESS as _]) -> bool {
    address[0] == 0xff
        && address[1] == 0xff
        && address[2] == 0xff
        && address[3] == 0xff
        && address[4] == 0xff
        && address[5] == 0xff
}
