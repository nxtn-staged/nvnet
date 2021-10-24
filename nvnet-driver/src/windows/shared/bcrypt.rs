use winapi::shared::bcrypt;

macro_rules! utf16_str {
    ($s:expr) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const LEN: usize = BYTES.len();
        let mut chars = [0u16; LEN + 1];
        let mut i = 0;
        while i < LEN {
            chars[i] = BYTES[i] as u16;
            i += 1;
        }
        chars[LEN] = 0;
        chars
    }};
}

// L235
pub use bcrypt::{
    BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO, BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
    BCRYPT_AUTH_MODE_CHAIN_CALLS_FLAG,
};

pub const BCRYPT_KEY_DATA_BLOB: [u16; 12] = utf16_str!(bcrypt::BCRYPT_KEY_DATA_BLOB);

pub const BCRYPT_CHAINING_MODE: [u16; 13] = utf16_str!(bcrypt::BCRYPT_CHAINING_MODE);

// L363
pub const BCRYPT_CHAIN_MODE_GCM: [u16; 16] = utf16_str!(bcrypt::BCRYPT_CHAIN_MODE_GCM);

// L650
pub use bcrypt::{
    BCRYPT_KEY_DATA_BLOB_HEADER, BCRYPT_KEY_DATA_BLOB_MAGIC, BCRYPT_KEY_DATA_BLOB_VERSION1,
};

// L809
pub const BCRYPT_AES_ALGORITHM: [u16; 4] = utf16_str!(bcrypt::BCRYPT_AES_ALGORITHM);

// L1038
pub use bcrypt::{
    BCryptDecrypt, BCryptEncrypt, BCryptImportKey, BCryptOpenAlgorithmProvider, BCryptSetProperty,
};
