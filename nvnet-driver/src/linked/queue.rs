use core::ptr::{self, NonNull};

use crate::linked::Next;

pub struct LinkedQueue<T: Next> {
    head: *mut T,
    tail: NonNull<*mut T>,
}

impl<T: Next> LinkedQueue<T> {
    pub unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        ptr::addr_of_mut!((*uninit).head).write(ptr::null_mut());
        ptr::addr_of_mut!((*uninit).tail).write((&mut (*uninit).head).into());
        &mut *uninit
    }

    pub unsafe fn chain(&mut self) -> *mut T {
        self.head
    }

    pub unsafe fn enqueue(&mut self, node: *mut T) {
        self.enqueue_fast(node, node);
    }

    pub unsafe fn enqueue_fast(&mut self, head: *mut T, tail: *mut T) {
        debug_assert!((*tail).next().is_null());
        *self.tail.as_mut() = head;
        self.tail = (*tail).next_mut().into();
    }

    pub fn dequeue(&mut self) -> Option<NonNull<T>> {
        let head = NonNull::new(self.head);
        if let Some(mut head) = head {
            unsafe {
                if self.tail == head.as_mut().next_mut().into() {
                    Self::init(self);
                } else {
                    self.head = head.as_ref().next();
                    *head.as_mut().next_mut() = ptr::null_mut();
                }
            }
        }
        head
    }

    pub fn dequeue_all(&mut self) -> *mut T {
        let head = self.head;
        unsafe { Self::init(self) };
        head
    }
}

pub struct LinkedCountedQueue<T: Next> {
    queue: LinkedQueue<T>,
    count: usize,
}

impl<T: Next> LinkedCountedQueue<T> {
    pub unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        LinkedQueue::init(ptr::addr_of_mut!((*uninit).queue));
        ptr::addr_of_mut!((*uninit).count).write(0);
        &mut *uninit
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub unsafe fn chain(&mut self) -> *mut T {
        self.queue.head
    }

    pub unsafe fn enqueue(&mut self, node: *mut T) {
        self.queue.enqueue(node);
        self.count += 1;
    }
}
