use core::{
    default::default,
    mem::{self, MaybeUninit},
    ptr,
};

use shared::crypto::{BCryptAlgHandle, BCryptKeyHandle};

use crate::windows::prelude as win;

pub struct AesGcm {
    key_handle: BCryptKeyHandle,
}

impl AesGcm {
    pub const KEY_SIZE128: usize = 128 / 8;
    pub const NONCE_SIZE12: usize = 12;
    pub const TAG_SIZE16: usize = 16;

    pub fn new(key: [u8; Self::KEY_SIZE128]) -> Result<Self, win::NTSTATUS> {
        let mut alg_handle = MaybeUninit::uninit();
        let status = unsafe {
            win::BCryptOpenAlgorithmProvider(
                alg_handle.as_mut_ptr(),
                win::BCRYPT_AES_ALGORITHM.as_ptr(),
                ptr::null(),
                0,
            )
            .into()
        };
        if status != win::STATUS_SUCCESS {
            return Err(status);
        }
        let alg_handle = unsafe { BCryptAlgHandle::from_raw(alg_handle.assume_init()) };
        let status = unsafe {
            win::BCryptSetProperty(
                alg_handle.as_raw().into(),
                win::BCRYPT_CHAINING_MODE.as_ptr(),
                win::BCRYPT_CHAIN_MODE_GCM.as_ptr() as *mut _,
                mem::size_of_val(&win::BCRYPT_CHAIN_MODE_GCM) as _,
                0,
            )
            .into()
        };
        if status != win::STATUS_SUCCESS {
            return Err(status);
        }
        let blob = BCryptKeyDataBlob {
            header: win::BCRYPT_KEY_DATA_BLOB_HEADER {
                dwMagic: win::BCRYPT_KEY_DATA_BLOB_MAGIC,
                dwVersion: win::BCRYPT_KEY_DATA_BLOB_VERSION1,
                cbKeyData: mem::size_of_val(&key) as _,
            },
            key,
        };
        let mut key_handle = MaybeUninit::uninit();
        let status = unsafe {
            win::BCryptImportKey(
                alg_handle.as_raw(),
                ptr::null_mut(),
                win::BCRYPT_KEY_DATA_BLOB.as_ptr(),
                key_handle.as_mut_ptr(),
                ptr::null_mut(),
                0,
                &blob as *const _ as *mut _,
                mem::size_of_val(&blob) as _,
                0,
            )
            .into()
        };
        if status != win::STATUS_SUCCESS {
            return Err(status);
        }
        let key_handle = unsafe { BCryptKeyHandle::from_raw(key_handle.assume_init()) };
        Ok(Self { key_handle })
    }

    pub fn encrypt(
        &self,
        nonce: &[u8],
        text: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), win::NTSTATUS> {
        let buf_size = mem::size_of_val(text) as _;
        let auth_info = win::BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
            pbNonce: nonce.as_ptr() as *mut _,
            cbNonce: mem::size_of_val(nonce) as _,
            pbTag: tag.as_mut_ptr(),
            cbTag: mem::size_of_val(tag) as _,
            ..default()
        };
        let mut written = MaybeUninit::uninit();
        let status = unsafe {
            win::BCryptEncrypt(
                self.key_handle.as_raw(),
                text.as_ptr() as *mut _,
                buf_size,
                &auth_info as *const _ as *mut _,
                ptr::null_mut(),
                0,
                text.as_mut_ptr(),
                buf_size,
                written.as_mut_ptr(),
                0,
            )
            .into()
        };
        let written = unsafe { written.assume_init() };
        debug_assert_eq!(buf_size, written);
        if status != win::STATUS_SUCCESS {
            return Err(status);
        }
        Ok(())
    }

    pub fn decrypt(&self, nonce: &[u8], text: &mut [u8], tag: &[u8]) -> Result<(), win::NTSTATUS> {
        let buf_size = mem::size_of_val(text) as _;
        let auth_info = win::BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
            pbNonce: nonce.as_ptr() as *mut _,
            cbNonce: mem::size_of_val(nonce) as _,
            pbTag: tag.as_ptr() as *mut _,
            cbTag: mem::size_of_val(tag) as _,
            ..default()
        };
        let mut written = MaybeUninit::uninit();
        let status = unsafe {
            win::BCryptDecrypt(
                self.key_handle.as_raw(),
                text.as_ptr() as *mut _,
                buf_size,
                &auth_info as *const _ as *mut _,
                ptr::null_mut(),
                0,
                text.as_mut_ptr(),
                buf_size,
                written.as_mut_ptr(),
                0,
            )
            .into()
        };
        let written = unsafe { written.assume_init() };
        debug_assert_eq!(buf_size, written);
        if status != win::STATUS_SUCCESS {
            return Err(status);
        }
        Ok(())
    }
}

#[repr(C)]
struct BCryptKeyDataBlob<T> {
    header: win::BCRYPT_KEY_DATA_BLOB_HEADER,
    key: T,
}
