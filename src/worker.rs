use core::{
    ptr,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use crate::{
    os::{event::AutoEvent, thread::Thread},
    windows::shared::ntstatus::STATUS_SUCCESS,
};

pub struct Worker {
    thread: Option<Thread>,
    state: WorkerState,
}

impl Worker {
    pub unsafe fn init<'a>(uninit: *mut Self) -> &'a mut WorkerState {
        ptr::raw_mut!((*uninit).thread).write(None);
        WorkerState::init(ptr::raw_mut!((*uninit).state))
    }

    pub fn init_thread(&mut self, thread: Thread) {
        self.thread.replace(thread).unwrap_none()
    }

    pub fn start(&mut self) {
        self.state.inner_stopping.store(false, Relaxed);
        self.state.outer.set();
    }

    pub fn terminate(&mut self) {
        self.state.outer_stopping.store(true, Relaxed);
        self.state.outer.set();
        self.thread.as_ref().unwrap().join();
    }

    pub fn signal_work(&mut self) {
        self.state.inner.set();
    }

    pub fn cancel(&mut self) {
        self.state.inner_stopping.store(true, Relaxed);
        self.signal_work();
    }

    pub fn wait_for_stopped(&mut self) {
        self.state.inner_stopped.wait();
    }
}

pub struct WorkerState {
    outer: AutoEvent,
    inner: AutoEvent,
    inner_stopped: AutoEvent,

    outer_stopping: AtomicBool,
    inner_stopping: AtomicBool,
}

impl WorkerState {
    unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        AutoEvent::init(ptr::raw_mut!((*uninit).outer));
        AutoEvent::init(ptr::raw_mut!((*uninit).inner));
        AutoEvent::init(ptr::raw_mut!((*uninit).inner_stopped));

        ptr::raw_mut!((*uninit).outer_stopping).write(AtomicBool::new(false));
        ptr::raw_mut!((*uninit).inner_stopping).write(AtomicBool::new(false));

        &mut *uninit
    }

    pub fn wait_for_start(&mut self) -> bool {
        self.outer.wait();
        !self.outer_stopping.load(Relaxed)
    }

    pub fn exit(&mut self) -> ! {
        Thread::exit(STATUS_SUCCESS);
    }

    pub fn is_canceled(&mut self) -> bool {
        self.inner_stopping.load(Relaxed)
    }

    pub fn wait_for_work(&mut self) -> bool {
        self.inner.wait();
        !self.is_canceled()
    }

    pub fn signal_stopped(&mut self) {
        self.inner_stopped.set();
    }
}
