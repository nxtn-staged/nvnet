use core::{
    cell::UnsafeCell,
    default::default,
    ops::{Deref, DerefMut},
};

use crate::windows::prelude as win;

pub struct RwLock<T> {
    lock: UnsafeCell<win::EX_SPIN_LOCK>,
    data: UnsafeCell<T>,
}

impl<T> RwLock<T> {
    pub fn read(&self) -> RwLockReadGuard<T> {
        let old_irql = unsafe { win::ExAcquireSpinLockShared(self.lock.get()) };
        RwLockReadGuard::new(self, old_irql)
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        let old_irql = unsafe { win::ExAcquireSpinLockExclusive(self.lock.get()) };
        RwLockWriteGuard::new(self, old_irql)
    }
}

impl<T: Default> Default for RwLock<T> {
    fn default() -> Self {
        Self {
            lock: default(),
            data: default(),
        }
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
    old_irql: win::KIRQL,
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
    old_irql: win::KIRQL,
}

impl<'a, T> RwLockReadGuard<'a, T> {
    fn new(lock: &'a RwLock<T>, old_irql: win::KIRQL) -> Self {
        Self { lock, old_irql }
    }
}

impl<'a, T> RwLockWriteGuard<'a, T> {
    fn new(lock: &'a RwLock<T>, old_irql: win::KIRQL) -> Self {
        Self { lock, old_irql }
    }
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { win::ExReleaseSpinLockShared(self.lock.lock.get(), self.old_irql) }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { win::ExReleaseSpinLockExclusive(self.lock.lock.get(), self.old_irql) }
    }
}
