use core::{
    default::default,
    mem::{self, MaybeUninit},
    ptr,
};

use crate::{
    debug::ResultExt,
    windows::{
        km::{
            ntifs::ZwWaitForSingleObject,
            wdm::{PsCreateSystemThread, PsTerminateSystemThread, ZwClose, THREAD_ALL_ACCESS},
        },
        shared::ntdef::{HANDLE, NTSTATUS},
        OkExt, Result,
    },
};

pub struct Thread(HANDLE);

impl Thread {
    pub fn spawn<'a, T>(
        start: extern "system" fn(&'a T) -> !,
        start_context: &'a T,
    ) -> Result<Self> {
        let mut handle = MaybeUninit::uninit();
        unsafe {
            PsCreateSystemThread(
                handle.as_mut_ptr(),
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                default(),
                ptr::null_mut(),
                Some(mem::transmute(start)),
                start_context as *const _ as *mut _,
            )
            .ok()
            .context_exit("PsCreateSystemThread")?
        };
        let handle = unsafe { handle.assume_init() };
        Ok(Self(handle))
    }

    pub fn join(&self) {
        let status = unsafe { ZwWaitForSingleObject(self.0, false, ptr::null_mut()) };
        debug_assert!(status.ok().is_ok());
    }

    pub fn exit(status: NTSTATUS) -> ! {
        let status = unsafe { PsTerminateSystemThread(status) };
        debug_assert!(status.ok().is_ok());
        unreachable!();
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let status = unsafe { ZwClose(self.0) };
        debug_assert!(status.ok().is_ok());
    }
}
