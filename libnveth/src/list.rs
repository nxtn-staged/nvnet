use core::{marker::PhantomData, mem};

use crate::{
    init::InitGuard,
    windows::{
        km::wdm::{
            ExAllocateFromLookasideListEx, ExDeleteLookasideListEx, ExFreeToLookasideListEx,
            ExInitializeLookasideListEx, LOOKASIDE_LIST_EX, POOL_TYPE,
        },
        shared::ntdef::{NTSTATUS, NT_SUCCESS},
    },
};

pub struct BufPool<T> {
    lookaside: LOOKASIDE_LIST_EX,
    t: PhantomData<T>,
}

impl<T> BufPool<T> {
    pub fn init(uninit: *mut Self, tag: u32) -> Result<InitGuard<Self>, NTSTATUS> {
        unsafe {
            let uninit = uninit.as_mut().unwrap();
            let status = ExInitializeLookasideListEx(
                &mut uninit.lookaside,
                None,
                None,
                POOL_TYPE::NonPagedPool,
                0,
                mem::size_of::<T>(),
                tag,
                0,
            );
            if !NT_SUCCESS(status) {
                Err(status)
            } else {
                Ok(InitGuard::new(uninit))
            }
        }
    }

    pub fn allocate(&mut self) -> *mut T {
        ExAllocateFromLookasideListEx(&mut self.lookaside).cast()
    }

    pub fn free(&mut self, entry: *mut T) {
        ExFreeToLookasideListEx(&mut self.lookaside, entry.cast())
    }
}

impl<T> Drop for BufPool<T> {
    fn drop(&mut self) {
        unsafe { ExDeleteLookasideListEx(&mut self.lookaside) }
    }
}
