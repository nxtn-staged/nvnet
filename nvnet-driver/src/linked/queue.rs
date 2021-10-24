use core::ptr;

use crate::linked::Next;

pub struct LinkedQueue<T: Next> {
    head: *mut T,
    tail: *mut *mut T,
}

impl<T: Next> LinkedQueue<T> {
    pub unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        ptr::addr_of_mut!((*uninit).head).write(ptr::null_mut());
        ptr::addr_of_mut!((*uninit).tail).write(&mut (*uninit).head);
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
        *self.tail = head;
        self.tail = (*tail).next_mut();
    }

    pub fn dequeue(&mut self) -> *mut T {
        if self.head.is_null() {
            ptr::null_mut()
        } else if self.tail == unsafe { (*self.head).next_mut() } {
            let head = self.head;
            unsafe { Self::init(self) };
            head
        } else {
            let head = self.head;
            self.head = unsafe { (*head).next() };
            unsafe { *(*head).next_mut() = ptr::null_mut() };
            head
        }
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
