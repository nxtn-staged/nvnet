use core::{
    alloc::Layout,
    cell::UnsafeCell,
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
};

use crate::{
    debug::ResultExt,
    windows::{
        km::{
            ntddk::ExFreePool,
            wdm::{
                ExAllocateFromLookasideListEx, ExAllocatePoolUninitialized,
                ExDeleteLookasideListEx, ExFreeToLookasideListEx, ExInitializeLookasideListEx,
                LOOKASIDE_LIST_EX, POOL_TYPE,
            },
        },
        shared::ntstatus::STATUS_INSUFFICIENT_RESOURCES,
        OkExt, Result,
    },
};

#[repr(transparent)]
pub struct Memory<T>(NonNull<T>);

impl<T> Memory<T> {
    pub fn alloc_non_paged(len: usize, tag: u32) -> Result<Self> {
        let layout = match Layout::array::<T>(len) {
            Err(_) => return Err(STATUS_INSUFFICIENT_RESOURCES),
            Ok(layout) => layout,
        };
        let ptr =
            unsafe { ExAllocatePoolUninitialized(POOL_TYPE::NonPagedPool, layout.size(), tag) };
        let ptr = NonNull::new(ptr)
            .ok_or(STATUS_INSUFFICIENT_RESOURCES)
            .context_exit("ExAllocatePoolUninitialized")?;
        Ok(Self(ptr.cast()))
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T> Drop for Memory<T> {
    fn drop(&mut self) {
        unsafe { ExFreePool(self.0.as_ptr().cast()) };
    }
}

#[repr(transparent)]
pub struct Lookaside<T> {
    lookaside: UnsafeCell<LOOKASIDE_LIST_EX>,
    marker: PhantomData<T>,
}

impl<T> Lookaside<T> {
    pub unsafe fn init<'a>(uninit: *mut Self, tag: u32) -> Result<&'a mut Self> {
        ExInitializeLookasideListEx(
            UnsafeCell::raw_get(ptr::addr_of!((*uninit).lookaside)),
            None,
            None,
            POOL_TYPE::NonPagedPool,
            0,
            mem::size_of::<T>(),
            tag,
            0,
        )
        .ok()
        .context_exit("ExInitializeLookasideListEx")?;
        Ok(&mut (*uninit))
    }

    pub unsafe fn alloc(&self) -> Result<NonNull<T>> {
        let ptr = ExAllocateFromLookasideListEx(self.lookaside.get());
        let ptr = NonNull::new(ptr)
            .ok_or(STATUS_INSUFFICIENT_RESOURCES)
            .context_exit("ExAllocateFromLookasideListEx")?;
        Ok(ptr.cast())
    }

    pub unsafe fn dealloc(&self, entry: *mut T) {
        ExFreeToLookasideListEx(self.lookaside.get(), entry.cast());
    }
}

impl<T> Drop for Lookaside<T> {
    fn drop(&mut self) {
        unsafe { ExDeleteLookasideListEx(self.lookaside.get()) };
    }
}
