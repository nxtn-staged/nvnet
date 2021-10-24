use core::marker::PhantomData;

use crate::linked::Next;

pub struct LinkedIter<'a, T: Next> {
    chain: *mut T,
    marker: PhantomData<&'a mut T>,
}

impl<T: Next> LinkedIter<'_, T> {
    pub unsafe fn new(chain: *mut T) -> Self {
        Self {
            chain,
            marker: PhantomData,
        }
    }
}

impl<T: Next> Clone for LinkedIter<'_, T> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<'a, T: Next> Iterator for LinkedIter<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = unsafe { self.chain.as_mut() };
        if let Some(cur) = cur.as_ref() {
            self.chain = cur.next();
        }
        cur
    }
}
