macro_rules! utf16 {
    ($bytes:expr) => {{
        const BYTES: &[u8] = $bytes;
        const LEN: usize = BYTES.len();
        let mut chars = [0u16; LEN];
        let mut i = 0;
        while i < LEN {
            chars[i] = BYTES[i] as _;
            i += 1;
        }
        chars
    }};
}
