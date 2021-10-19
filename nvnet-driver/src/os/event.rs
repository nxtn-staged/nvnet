use core::{cell::UnsafeCell, ptr};

use crate::windows::{
    km::wdm::{
        KeInitializeEvent, KeSetEvent, KeWaitForSingleObject, IO_NO_INCREMENT, KEVENT,
        KPROCESSOR_MODE, KWAIT_REASON,
    },
    shared::ntdef::EVENT_TYPE,
    OkExt,
};

pub struct AutoEvent(UnsafeCell<KEVENT>);

impl AutoEvent {
    pub unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        KeInitializeEvent(
            UnsafeCell::raw_get(ptr::addr_of!((*uninit).0)),
            EVENT_TYPE::SynchronizationEvent,
            false,
        );
        &mut *uninit
    }

    pub fn wait(&self) {
        let status = unsafe {
            KeWaitForSingleObject(
                self.0.get() as *mut _,
                KWAIT_REASON::Executive,
                KPROCESSOR_MODE::KernelMode,
                false,
                ptr::null_mut(),
            )
        };
        debug_assert!(status.ok().is_ok());
    }

    pub fn set(&self) {
        unsafe { KeSetEvent(self.0.get(), IO_NO_INCREMENT.into(), false) };
    }
}
