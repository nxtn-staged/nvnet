use core::{mem::MaybeUninit, ptr};

pub struct InitGuard<T: ?Sized>(*mut T);

impl<T> InitGuard<T> {
    pub fn new(init: *mut T) -> Self {
        Self(init)
    }
}

impl<T: ?Sized> Drop for InitGuard<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.0) }
    }
}

pub struct ManuallyInit<T>(Option<MaybeUninit<T>>);

impl<T> ManuallyInit<T> {
    pub const fn uninit() -> Self {
        Self(None)
    }

    pub fn init<F, U>(&mut self, init: F) -> U
    where
        F: FnOnce(*mut T) -> U,
    {
        self.0.replace(MaybeUninit::uninit()).unwrap_none();
        init(self.0.as_mut().unwrap().as_mut_ptr())
    }

    pub fn get(&self) -> &T {
        unsafe { self.0.as_ref().unwrap().assume_init_ref() }
    }

    pub fn drop(&mut self) {
        unsafe { self.0.as_mut().unwrap().assume_init_drop() };
        self.0 = None;
    }
}
