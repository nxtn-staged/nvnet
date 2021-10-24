use core::{
    default::default,
    mem::{self, MaybeUninit},
    ptr,
};

use nvnet_shared::crypto::{BCryptAlgHandle, BCryptKeyHandle};

use crate::{
    debug::ResultExt,
    windows::{
        shared::bcrypt::{
            BCryptDecrypt, BCryptEncrypt, BCryptImportKey, BCryptOpenAlgorithmProvider,
            BCryptSetProperty, BCRYPT_AES_ALGORITHM, BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO,
            BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION, BCRYPT_AUTH_MODE_CHAIN_CALLS_FLAG,
            BCRYPT_CHAINING_MODE, BCRYPT_CHAIN_MODE_GCM, BCRYPT_KEY_DATA_BLOB,
            BCRYPT_KEY_DATA_BLOB_HEADER, BCRYPT_KEY_DATA_BLOB_MAGIC, BCRYPT_KEY_DATA_BLOB_VERSION1,
        },
        OkExt, Result,
    },
};

pub struct AesGcm(BCryptKeyHandle);

impl AesGcm {
    pub fn new(key: [u8; 16]) -> Result<Self> {
        let mut alg_handle = MaybeUninit::uninit();
        unsafe {
            BCryptOpenAlgorithmProvider(
                alg_handle.as_mut_ptr(),
                BCRYPT_AES_ALGORITHM.as_ptr(),
                ptr::null(),
                0,
            )
            .ok()
            .context_exit("BCryptOpenAlgorithmProvider")?
        };
        let alg_handle = unsafe { BCryptAlgHandle::from_raw(alg_handle.assume_init()) };
        unsafe {
            BCryptSetProperty(
                alg_handle.as_raw(),
                BCRYPT_CHAINING_MODE.as_ptr(),
                BCRYPT_CHAIN_MODE_GCM.as_ptr() as *mut _,
                mem::size_of_val(&BCRYPT_CHAIN_MODE_GCM) as u32,
                0,
            )
            .ok()
            .context_exit("BCryptSetProperty")?
        };
        let mut blob = BCryptKeyDataBlob {
            header: BCRYPT_KEY_DATA_BLOB_HEADER {
                dwMagic: BCRYPT_KEY_DATA_BLOB_MAGIC,
                dwVersion: BCRYPT_KEY_DATA_BLOB_VERSION1,
                cbKeyData: mem::size_of_val(&key) as u32,
            },
            key,
        };
        let mut key_handle = MaybeUninit::uninit();
        unsafe {
            BCryptImportKey(
                alg_handle.as_raw(),
                ptr::null_mut(),
                BCRYPT_KEY_DATA_BLOB.as_ptr(),
                key_handle.as_mut_ptr(),
                ptr::null_mut(),
                0,
                &mut blob as *mut _ as *mut _,
                mem::size_of_val(&blob) as u32,
                0,
            )
            .ok()
            .context_exit("BCryptImportKey")?
        };
        let key_handle = unsafe { BCryptKeyHandle::from_raw(key_handle.assume_init()) };
        Ok(Self(key_handle))
    }

    pub fn encrypt_chain<'a>(
        &self,
        nonce: &[u8; 12],
        text_iter: impl Iterator<Item = Result<&'a mut [u8]>>,
        tag: &mut [u8; 16],
    ) -> Result<()> {
        let mut ctx = [0; 16];
        let mut auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
            cbSize: mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
            dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
            pbNonce: nonce.as_ptr() as *mut _,
            cbNonce: mem::size_of_val(nonce) as u32,
            pbTag: tag.as_mut_ptr(),
            cbTag: mem::size_of_val(tag) as u32,
            pbMacContext: ctx.as_mut_ptr(),
            cbMacContext: mem::size_of_val(&ctx) as u32,
            ..default()
        };
        let mut text_iter = text_iter.peekable();
        while let Some(text) = text_iter.next() {
            let text = text?;
            auth_info.dwFlags = if text_iter.peek().is_some() {
                BCRYPT_AUTH_MODE_CHAIN_CALLS_FLAG
            } else {
                0
            };
            let text_size = mem::size_of_val(&text) as u32;
            let mut written = MaybeUninit::uninit();
            unsafe {
                BCryptEncrypt(
                    self.0.as_raw(),
                    text.as_mut_ptr(),
                    text_size,
                    &mut auth_info as *mut _ as *mut _,
                    ptr::null_mut(),
                    0,
                    text.as_mut_ptr(),
                    text_size,
                    written.as_mut_ptr(),
                    0,
                )
                .ok()
                .context_exit("BCryptEncrypt")?
            };
            let written = unsafe { written.assume_init() };
            debug_assert_eq!(written, text_size);
        }
        Ok(())
    }

    pub fn decrypt_chain<'a>(
        &self,
        nonce: &[u8; 12],
        text_iter: impl Iterator<Item = Result<&'a mut [u8]>>,
        tag: &mut [u8; 16],
    ) -> Result<()> {
        let mut ctx = [0; 16];
        let mut auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
            cbSize: mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
            dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
            pbNonce: nonce.as_ptr() as *mut _,
            cbNonce: mem::size_of_val(nonce) as u32,
            pbTag: tag.as_mut_ptr(),
            cbTag: mem::size_of_val(tag) as u32,
            pbMacContext: ctx.as_mut_ptr(),
            cbMacContext: mem::size_of_val(&ctx) as u32,
            ..default()
        };
        let mut text_iter = text_iter.peekable();
        while let Some(text) = text_iter.next() {
            let text = text?;
            auth_info.dwFlags = if text_iter.peek().is_some() {
                BCRYPT_AUTH_MODE_CHAIN_CALLS_FLAG
            } else {
                0
            };
            let text_size = mem::size_of_val(&text) as u32;
            let mut written = MaybeUninit::uninit();
            unsafe {
                BCryptDecrypt(
                    self.0.as_raw(),
                    text.as_mut_ptr(),
                    text_size,
                    &mut auth_info as *mut _ as *mut _,
                    ptr::null_mut(),
                    0,
                    text.as_mut_ptr(),
                    text_size,
                    written.as_mut_ptr(),
                    0,
                )
                .ok()
                .context_exit("BCryptDecrypt")?
            };
            let written = unsafe { written.assume_init() };
            debug_assert_eq!(written, text_size);
        }
        Ok(())
    }
}

#[repr(C)]
struct BCryptKeyDataBlob {
    header: BCRYPT_KEY_DATA_BLOB_HEADER,
    key: [u8; 16],
}
