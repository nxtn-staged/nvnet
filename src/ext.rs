pub trait AsPtrExt<T> {
    unsafe fn unsafe_as<U>(&self) -> &U;

    fn as_ptr(&self) -> *const Self;

    fn as_mut_ptr(&mut self) -> *mut Self;

    fn to_mut_ptr(&self) -> *mut Self;
}

impl<T> AsPtrExt<T> for T {
    unsafe fn unsafe_as<U>(&self) -> &U {
        &*self.as_ptr().cast()
    }

    fn as_ptr(&self) -> *const Self {
        self as *const T
    }

    fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut T
    }

    fn to_mut_ptr(&self) -> *mut Self {
        self as *const T as *mut T
    }
}
