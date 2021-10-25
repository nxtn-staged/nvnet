use core::{cell::UnsafeCell, ffi::c_void, mem, ptr, slice};

use crate::{
    os::event::AutoEvent,
    windows::{
        km::wdm::{
            IoCancelIrp, IoCompleteRequest, IoGetCurrentIrpStackLocation, IoInitializeIrp,
            IoReuseIrp, IoSetCompletionRoutine, DEVICE_OBJECT, IO_NO_INCREMENT, IO_STACK_LOCATION,
            IRP,
        },
        shared::{
            ntdef::NTSTATUS,
            ntstatus::{
                STATUS_INVALID_BUFFER_SIZE, STATUS_MORE_PROCESSING_REQUIRED, STATUS_PENDING,
                STATUS_SUCCESS,
            },
        },
        OkExt, Result,
    },
};

pub struct Request;

impl Request {
    pub unsafe fn complete(irp: *mut IRP, status: NTSTATUS) {
        (*irp).IoStatus.Status = status;
        IoCompleteRequest(irp, IO_NO_INCREMENT);
    }

    pub unsafe fn retrieve_input_buf<'a>(irp: *mut IRP) -> &'a mut [u8] {
        let irp_sp = IoGetCurrentIrpStackLocation(irp);
        slice::from_raw_parts_mut(
            (*irp).SystemBuffer.cast(),
            (*irp_sp).Parameters.InputBufferLength as usize,
        )
    }

    pub unsafe fn retrieve_input_val<T>(irp: *mut IRP) -> Result<*mut T> {
        let buf = Self::retrieve_input_buf(irp);
        if buf.len() < mem::size_of::<T>() {
            return Err(STATUS_INVALID_BUFFER_SIZE);
        }
        Ok(buf.as_mut_ptr().cast())
    }
}

pub struct SyncRequest {
    irp: UnsafeCell<IrpRepr<1>>,
    event: AutoEvent,
}

impl SyncRequest {
    pub unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        IrpRepr::init(UnsafeCell::raw_get(ptr::addr_of!((*uninit).irp)));
        AutoEvent::init(ptr::addr_of_mut!((*uninit).event));
        &mut *uninit
    }

    pub fn invoke(&self, invoke: impl FnOnce(*mut IRP) -> NTSTATUS) -> Result<()> {
        let irp = unsafe { self.irp.get().raw_get() };
        unsafe { IoReuseIrp(irp, STATUS_SUCCESS) };
        unsafe {
            IoSetCompletionRoutine(
                irp,
                Some(Self::complete),
                &self.event as *const _ as *mut _,
                true,
                true,
                true,
            )
        };
        let status = invoke(irp);
        let status = if status != STATUS_PENDING {
            status
        } else {
            self.event.wait();
            unsafe { (*irp).IoStatus.Status }
        };
        status.ok()
    }

    pub fn info(&self) -> usize {
        let irp = unsafe { self.irp.get().raw_get() };
        unsafe { (*irp).IoStatus.Information }
    }

    pub fn cancel(&self) {
        let irp = unsafe { self.irp.get().raw_get() };
        unsafe { IoCancelIrp(irp) };
    }

    extern "system" fn complete(
        _device_object: *mut DEVICE_OBJECT,
        _irp: *mut IRP,
        context: *mut c_void,
    ) -> NTSTATUS {
        let event = unsafe { &*context.cast::<AutoEvent>() };
        event.set();
        STATUS_MORE_PROCESSING_REQUIRED
    }
}

#[repr(C)]
struct IrpRepr<const N: usize> {
    irp: IRP,
    _irpx: [IO_STACK_LOCATION; N],
}

impl<const N: usize> IrpRepr<N> {
    unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        IoInitializeIrp(uninit.raw_get(), mem::size_of::<Self>() as u16, N as i8);
        &mut *uninit
    }

    unsafe fn raw_get(self: *mut Self) -> *mut IRP {
        ptr::addr_of_mut!((*self).irp)
    }
}
