use winapi::shared::{
    bcrypt::{
        BCryptCloseAlgorithmProvider, BCryptDestroyKey, BCryptDestroySecret, BCRYPT_ALG_HANDLE,
        BCRYPT_KEY_HANDLE, BCRYPT_SECRET_HANDLE,
    },
    ntstatus::STATUS_SUCCESS,
};

macro_rules! win_wrapper {
    ($vis:vis struct $name:ident($ty:ty);) => {
        $vis struct $name($ty);

        impl $name {
            pub fn from_raw(handle: $ty) -> Self {
                Self(handle)
            }

            pub fn as_raw(&self) -> $ty {
                self.0
            }
        }
    };
}

win_wrapper!(
    pub struct BCryptAlgHandle(BCRYPT_ALG_HANDLE);
);

impl Drop for BCryptAlgHandle {
    fn drop(&mut self) {
        let status = unsafe { BCryptCloseAlgorithmProvider(self.as_raw(), 0) };
        debug_assert_eq!(status, STATUS_SUCCESS);
    }
}

win_wrapper!(
    pub struct BCryptKeyHandle(BCRYPT_KEY_HANDLE);
);

impl Drop for BCryptKeyHandle {
    fn drop(&mut self) {
        let status = unsafe { BCryptDestroyKey(self.as_raw()) };
        debug_assert_eq!(status, STATUS_SUCCESS);
    }
}

win_wrapper!(
    pub struct BCryptSecretHandle(BCRYPT_SECRET_HANDLE);
);

impl Drop for BCryptSecretHandle {
    fn drop(&mut self) {
        let status = unsafe { BCryptDestroySecret(self.as_raw()) };
        debug_assert_eq!(status, STATUS_SUCCESS);
    }
}
