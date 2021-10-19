mod iter;
mod queue;

pub use self::{iter::LinkedIter, queue::LinkedQueue};

use crate::windows::km::{
    ndis::nbl::{NET_BUFFER, NET_BUFFER_LIST},
    wsk::WSK_BUF_LIST,
};

pub trait Next {
    fn next(&self) -> *mut Self;

    fn next_mut(&mut self) -> &mut *mut Self;
}

impl Next for NET_BUFFER {
    fn next(&self) -> *mut Self {
        self.next
    }

    fn next_mut(&mut self) -> &mut *mut Self {
        &mut self.next
    }
}

impl Next for NET_BUFFER_LIST {
    fn next(&self) -> *mut Self {
        self.next
    }

    fn next_mut(&mut self) -> &mut *mut Self {
        &mut self.next
    }
}

impl Next for WSK_BUF_LIST {
    fn next(&self) -> *mut Self {
        self.Next
    }

    fn next_mut(&mut self) -> &mut *mut Self {
        &mut self.Next
    }
}