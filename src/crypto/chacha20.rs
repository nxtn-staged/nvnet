pub struct ChaCha20([u32; 16]);

impl ChaCha20 {
    pub fn new(key: &[u8; 32]) -> Self {
        let mut state = [0u32; 16];

        state[0] = 0x61707865;
        state[1] = 0x3320646e;
        state[2] = 0x79622d32;
        state[3] = 0x6b206574;

        state[4] = u32::from_le_bytes([key[0], key[1], key[2], key[3]]);
        state[5] = u32::from_le_bytes([key[4], key[5], key[6], key[7]]);
        state[6] = u32::from_le_bytes([key[8], key[9], key[10], key[11]]);
        state[7] = u32::from_le_bytes([key[12], key[13], key[14], key[15]]);
        state[8] = u32::from_le_bytes([key[16], key[17], key[18], key[19]]);
        state[9] = u32::from_le_bytes([key[20], key[21], key[22], key[23]]);
        state[10] = u32::from_le_bytes([key[24], key[25], key[26], key[27]]);
        state[11] = u32::from_le_bytes([key[28], key[29], key[30], key[31]]);

        Self(state)
    }
}
