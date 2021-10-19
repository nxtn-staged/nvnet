use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr,
};

use crate::windows::{
    km::wdm::{
        KeAcquireSpinLock, KeAcquireSpinLockAtDpcLevel, KeInitializeSpinLock, KeReleaseSpinLock,
        KeReleaseSpinLockFromDpcLevel, KSPIN_LOCK,
    },
    shared::ntdef::KIRQL,
};

pub struct SpinLock<T> {
    lock: UnsafeCell<KSPIN_LOCK>,
    data: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    pub unsafe fn init<'a>(
        uninit: *mut Self,
        init: unsafe fn(*mut T) -> &'a mut T,
    ) -> &'a mut Self {
        KeInitializeSpinLock(UnsafeCell::raw_get(ptr::addr_of!((*uninit).lock)));
        init(UnsafeCell::raw_get(ptr::addr_of!((*uninit).data)));
        &mut *uninit
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        self.lock_fast(false)
    }

    pub fn lock_fast(&self, at_dispatch: bool) -> SpinLockGuard<T> {
        let lock = self.lock.get();
        let irql = unsafe {
            if at_dispatch {
                KeAcquireSpinLockAtDpcLevel(lock);
                None
            } else {
                let mut irql = MaybeUninit::uninit();
                KeAcquireSpinLock(lock, irql.as_mut_ptr());
                Some(irql.assume_init())
            }
        };
        SpinLockGuard { lock: self, irql }
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
    irql: Option<KIRQL>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        let lock = self.lock.lock.get();
        unsafe {
            match self.irql {
                None => KeReleaseSpinLockFromDpcLevel(lock),
                Some(irql) => KeReleaseSpinLock(lock, irql),
            }
        };
    }
}
