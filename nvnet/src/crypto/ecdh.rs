use std::{
    mem::{self, MaybeUninit},
    ptr,
};

use winapi::shared::{
    bcrypt::{
        BCryptBuffer, BCryptBufferDesc, BCryptDeriveKey, BCryptExportKey, BCryptFinalizeKeyPair,
        BCryptGenerateKeyPair, BCryptImportKeyPair, BCryptOpenAlgorithmProvider,
        BCryptSecretAgreement, BCryptSetProperty, BCRYPTBUFFER_VERSION, BCRYPT_ECCPRIVATE_BLOB,
        BCRYPT_ECCPUBLIC_BLOB, BCRYPT_ECC_CURVE_25519, BCRYPT_ECC_CURVE_NAME,
        BCRYPT_ECDH_ALGORITHM, BCRYPT_KDF_HASH, BCRYPT_SHA256_ALGORITHM, KDF_HASH_ALGORITHM,
    },
    ntstatus::STATUS_SUCCESS,
};

use shared::crypto::{BCryptAlgHandle, BCryptKeyHandle, BCryptSecretHandle};

use crate::error::WinError;

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

const U16_BCRYPT_KDF_HASH: [u16; 5] = utf16_str!(BCRYPT_KDF_HASH);
const U16_BCRYPT_ECCPRIVATE_BLOB: [u16; 15] = utf16_str!(BCRYPT_ECCPRIVATE_BLOB);
const U16_BCRYPT_ECCPUBLIC_BLOB: [u16; 14] = utf16_str!(BCRYPT_ECCPUBLIC_BLOB);
const U16_BCRYPT_ECC_CURVE_NAME: [u16; 13] = utf16_str!(BCRYPT_ECC_CURVE_NAME);
const U16_BCRYPT_ECC_CURVE_25519: [u16; 11] = utf16_str!(BCRYPT_ECC_CURVE_25519);
const U16_BCRYPT_SHA256_ALGORITHM: [u16; 7] = utf16_str!(BCRYPT_SHA256_ALGORITHM);
const U16_BCRYPT_ECDH_ALGORITHM: [u16; 5] = utf16_str!(BCRYPT_ECDH_ALGORITHM);

fn open_algorithm_provider() -> Result<BCryptAlgHandle, WinError> {
    let mut alg_handle = MaybeUninit::uninit();
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            alg_handle.as_mut_ptr(),
            U16_BCRYPT_ECDH_ALGORITHM.as_ptr(),
            ptr::null(),
            0,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(WinError::from_nt_status(status));
    }
    let alg_handle = unsafe { BCryptAlgHandle::from_raw(alg_handle.assume_init()) };
    let status = unsafe {
        BCryptSetProperty(
            alg_handle.as_raw(),
            U16_BCRYPT_ECC_CURVE_NAME.as_ptr(),
            U16_BCRYPT_ECC_CURVE_25519.as_ptr() as *mut _,
            mem::size_of_val(&U16_BCRYPT_ECC_CURVE_25519) as _,
            0,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(WinError::from_nt_status(status));
    }
    Ok(alg_handle)
}

pub struct Ecdh {
    key_handle: BCryptKeyHandle,
}

impl Ecdh {
    pub fn new() -> Result<Self, WinError> {
        let alg_handle = open_algorithm_provider()?;
        let mut key_handle = MaybeUninit::uninit();
        let status =
            unsafe { BCryptGenerateKeyPair(alg_handle.as_raw(), key_handle.as_mut_ptr(), 255, 0) };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        let key_handle = unsafe { BCryptKeyHandle::from_raw(key_handle.assume_init()) };
        let status = unsafe { BCryptFinalizeKeyPair(key_handle.as_raw(), 0) };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        Ok(Self { key_handle })
    }

    fn export_slice<'a>(
        &self,
        blob_format: &'static [u16],
        buf: &'a mut [u8],
    ) -> Result<&'a mut [u8], WinError> {
        let mut key_blob_size = MaybeUninit::uninit();
        let status = unsafe {
            BCryptExportKey(
                self.key_handle.as_raw(),
                ptr::null_mut(),
                blob_format.as_ptr(),
                buf.as_mut_ptr(),
                mem::size_of_val(buf) as _,
                key_blob_size.as_mut_ptr(),
                0,
            )
        };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        let key_blob_size = unsafe { key_blob_size.assume_init() };
        let (key_blob, _) = buf.split_at_mut(key_blob_size as _);
        Ok(key_blob)
    }

    pub fn export_private_key_slice<'a>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<&'a mut [u8], WinError> {
        self.export_slice(&U16_BCRYPT_ECCPRIVATE_BLOB, buf)
    }

    pub fn export_public_key_slice<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], WinError> {
        self.export_slice(&U16_BCRYPT_ECCPUBLIC_BLOB, buf)
    }

    pub fn derive_key(&self, other_pub_key: &EcdhPubKey) -> Result<Vec<u8>, WinError> {
        let mut secret_handle = MaybeUninit::uninit();
        let status = unsafe {
            BCryptSecretAgreement(
                self.key_handle.as_raw(),
                other_pub_key.key_handle.as_raw(),
                secret_handle.as_mut_ptr(),
                0,
            )
        };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        let secret_handle = unsafe { BCryptSecretHandle::from_raw(secret_handle.assume_init()) };
        let mut param = BCryptBuffer {
            cbBuffer: mem::size_of_val(&U16_BCRYPT_SHA256_ALGORITHM) as _,
            BufferType: KDF_HASH_ALGORITHM,
            pvBuffer: U16_BCRYPT_SHA256_ALGORITHM.as_ptr() as *mut _,
        };
        let mut param_desc = BCryptBufferDesc {
            ulVersion: BCRYPTBUFFER_VERSION,
            cBuffers: 1,
            pBuffers: &mut param,
        };
        let mut key_size = MaybeUninit::uninit();
        let status = unsafe {
            BCryptDeriveKey(
                secret_handle.as_raw(),
                U16_BCRYPT_KDF_HASH.as_ptr(),
                &mut param_desc,
                ptr::null_mut(),
                0,
                key_size.as_mut_ptr(),
                0,
            )
        };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        let mut key_size = unsafe { key_size.assume_init() };
        let mut key = vec![0; key_size as _];
        let status = unsafe {
            BCryptDeriveKey(
                secret_handle.as_raw(),
                U16_BCRYPT_KDF_HASH.as_ptr(),
                &mut param_desc,
                key.as_mut_ptr(),
                key_size,
                &mut key_size,
                0,
            )
        };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        Ok(key)
    }

    pub fn import(key_blob: &[u8]) -> Result<Self, WinError> {
        let alg_handle = open_algorithm_provider()?;
        let mut key_handle = MaybeUninit::uninit();
        let status = unsafe {
            BCryptImportKeyPair(
                alg_handle.as_raw(),
                ptr::null_mut(),
                U16_BCRYPT_ECCPRIVATE_BLOB.as_ptr(),
                key_handle.as_mut_ptr(),
                key_blob.as_ptr() as *mut _,
                mem::size_of_val(key_blob) as _,
                0,
            )
        };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        let key_handle = unsafe { BCryptKeyHandle::from_raw(key_handle.assume_init()) };
        Ok(Self { key_handle })
    }
}

pub struct EcdhPubKey {
    key_handle: BCryptKeyHandle,
}

impl EcdhPubKey {
    pub fn import(key_blob: &[u8]) -> Result<Self, WinError> {
        let alg_handle = open_algorithm_provider()?;
        let mut key_handle = MaybeUninit::uninit();
        let status = unsafe {
            BCryptImportKeyPair(
                alg_handle.as_raw(),
                ptr::null_mut(),
                U16_BCRYPT_ECCPUBLIC_BLOB.as_ptr(),
                key_handle.as_mut_ptr(),
                key_blob.as_ptr() as *mut _,
                mem::size_of_val(key_blob) as _,
                0,
            )
        };
        if status != STATUS_SUCCESS {
            return Err(WinError::from_nt_status(status));
        }
        let key_handle = unsafe { BCryptKeyHandle::from_raw(key_handle.assume_init()) };
        Ok(Self { key_handle })
    }
}
