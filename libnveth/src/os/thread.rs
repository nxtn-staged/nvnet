use core::{
    default::default,
    mem::{self, MaybeUninit},
    ptr, unreachable,
};

use crate::windows::{
    km::{
        ntifs::ZwWaitForSingleObject,
        wdm::{
            PsCreateSystemThread, PsTerminateSystemThread, ZwClose, PKSTART_ROUTINE,
            THREAD_ALL_ACCESS,
        },
    },
    shared::{
        ntdef::{HANDLE, NTSTATUS, NT_SUCCESS, PVOID},
        ntstatus::STATUS_SUCCESS,
    },
};

#[derive(Debug)]
pub struct Thread(HANDLE);

impl Thread {
    pub fn spawn(start: PKSTART_ROUTINE, start_context: PVOID) -> Result<Self, NTSTATUS> {
        let mut thread_handle = MaybeUninit::uninit();
        let status = unsafe {
            PsCreateSystemThread(
                thread_handle.as_mut_ptr(),
                THREAD_ALL_ACCESS,
                ptr::null(),
                default(),
                ptr::null_mut(),
                start,
                start_context,
            )
        };
        if !NT_SUCCESS(status) {
            Err(status)
        } else {
            let thread_handle = unsafe { thread_handle.assume_init() };
            Ok(Self(thread_handle))
        }
    }

    pub fn spawn_mut<T>(
        start: extern "system" fn(start_context: &mut T),
        start_context: &'static mut T,
    ) -> Result<Self, NTSTATUS> {
        unsafe {
            Self::spawn(
                Some(mem::transmute(start)),
                (start_context as *mut T).cast(),
            )
        }
    }

    pub fn exit(status: NTSTATUS) -> ! {
        unsafe { PsTerminateSystemThread(status) };
        unreachable!();
    }

    pub fn join(&self) {
        let status = unsafe { ZwWaitForSingleObject(self.0, false, ptr::null()) };
        assert_eq!(status, STATUS_SUCCESS);
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let status = unsafe { ZwClose(self.0) };
        debug_assert_eq!(status, STATUS_SUCCESS);
    }
}
