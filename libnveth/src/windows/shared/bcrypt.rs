use winapi::shared::bcrypt;

macro_rules! utf16_str {
    ($s:expr) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const LEN: usize = BYTES.len();
        let mut chars = [0u16; LEN + 1];
        let mut i = 0;
        while i < LEN {
            chars[i] = BYTES[i] as _;
            i += 1;
        }
        chars[LEN] = 0;
        chars
    }};
}

pub use bcrypt::BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO;

pub const BCRYPT_KEY_DATA_BLOB: [u16; 12] = utf16_str!(bcrypt::BCRYPT_KEY_DATA_BLOB);

pub const BCRYPT_CHAINING_MODE: [u16; 13] = utf16_str!(bcrypt::BCRYPT_CHAINING_MODE);

pub const BCRYPT_CHAIN_MODE_GCM: [u16; 16] = utf16_str!(bcrypt::BCRYPT_CHAIN_MODE_GCM);

pub use bcrypt::BCRYPT_KEY_DATA_BLOB_HEADER;
pub use bcrypt::BCRYPT_KEY_DATA_BLOB_MAGIC;
pub use bcrypt::BCRYPT_KEY_DATA_BLOB_VERSION1;

pub const BCRYPT_AES_ALGORITHM: [u16; 4] = utf16_str!(bcrypt::BCRYPT_AES_ALGORITHM);

pub use bcrypt::BCryptDecrypt;
pub use bcrypt::BCryptEncrypt;
pub use bcrypt::BCryptImportKey;
pub use bcrypt::BCryptOpenAlgorithmProvider;
pub use bcrypt::BCryptSetProperty;
