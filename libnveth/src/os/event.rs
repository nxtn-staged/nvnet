use core::ptr;

use crate::windows::{
    km::{
        ntddk::IO_NO_INCREMENT,
        wdm::{
            KeInitializeEvent, KeSetEvent, KeWaitForSingleObject, KEVENT, KPROCESSOR_MODE,
            KWAIT_REASON,
        },
    },
    shared::{ntdef::EVENT_TYPE, ntstatus::STATUS_SUCCESS},
};

pub struct AutoEvent(KEVENT);

impl AutoEvent {
    pub unsafe fn init(uninit: *mut Self) {
        KeInitializeEvent(
            ptr::raw_mut!((*uninit).0),
            EVENT_TYPE::SynchronizationEvent,
            false,
        )
    }

    pub fn wait(&self) {
        let status = unsafe {
            KeWaitForSingleObject(
                &self.0 as *const _ as *mut _,
                KWAIT_REASON::Executive,
                KPROCESSOR_MODE::KernelMode,
                false,
                ptr::null(),
            )
        };
        assert_eq!(status, STATUS_SUCCESS);
    }

    pub fn set(&self) {
        unsafe { KeSetEvent(&self.0 as *const _ as *mut _, IO_NO_INCREMENT.into(), false) };
    }
}
