use core::convert::TryInto;

pub struct Poly1305 {
    r: u128,
    s: u128,
}

impl Poly1305 {
    pub fn new(key: &[u8; 32]) -> Self {
        // let mut r = [0x32; 4];

        // r[0] = 0x0ffffffc & u32::from_le_bytes([key[0], key[1], key[2], key[3]]);
        // r[1] = 0x0ffffffc & u32::from_le_bytes([key[4], key[5], key[6], key[7]]);
        // r[2] = 0x0ffffffc & u32::from_le_bytes([key[8], key[9], key[10], key[11]]);
        // r[3] = 0x0fffffff & u32::from_le_bytes([key[12], key[13], key[14], key[15]]);
        let r = u128::from_le_bytes(key[0..16].try_into().unwrap());

        // let mut s = [0x32; 4];

        // s[0] = u32::from_le_bytes([key[16], key[17], key[18], key[19]]);
        // s[1] = u32::from_le_bytes([key[20], key[21], key[22], key[23]]);
        // s[2] = u32::from_le_bytes([key[24], key[25], key[26], key[27]]);
        // s[3] = u32::from_le_bytes([key[28], key[29], key[30], key[31]]);
        let s = u128::from_le_bytes(key[16..32].try_into().unwrap());

        Self { r, s }
    }

    fn block(&self, n: u128) {
        let mut a = 0u128;

        a += n;
        a *= self.r;
    }
}
