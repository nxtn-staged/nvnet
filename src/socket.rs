use core::{
    default::default,
    mem::{self, MaybeUninit},
    ptr,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use crate::{
    ext::AsPtrExt,
    init::{InitGuard, ManuallyInit},
    os::event::AutoEvent,
    windows::{
        km::{
            wdm::{
                IoAllocateIrp, IoCancelIrp, IoFreeIrp, IoReuseIrp, IoSetCompletionRoutine,
                DEVICE_OBJECT, IRP,
            },
            wsk as win,
        },
        shared::{
            ntdef::{NTSTATUS, NT_SUCCESS, PVOID},
            ntstatus::{
                STATUS_CANCELLED, STATUS_INSUFFICIENT_RESOURCES, STATUS_MORE_PROCESSING_REQUIRED,
                STATUS_PENDING, STATUS_SUCCESS,
            },
            ws2def::{AF_INET6, IPPROTO, SOCK_DGRAM},
            ws2ipdef::SOCKADDR_IN6,
        },
    },
};

// resident
static CLIENT_DISPATCH: win::WSK_CLIENT_DISPATCH = win::WSK_CLIENT_DISPATCH {
    version: win::MAKE_WSK_VERSION(1, 0),
    reserved: 0,
    wsk_client_event: None,
};

// resident
static mut REGISTRATION: ManuallyInit<SocketRegistration> = ManuallyInit::uninit();

struct SocketRegistration(win::WSK_REGISTRATION);

impl SocketRegistration {
    unsafe fn init(uninit: *mut Self) -> Result<(), NTSTATUS> {
        let client_npi = win::WSK_CLIENT_NPI {
            dispatch: &CLIENT_DISPATCH,
            ..default()
        };
        let status = win::WskRegister(&client_npi, ptr::raw_mut!((*uninit).0));
        if !NT_SUCCESS(status) {
            trace_exit_status_unsafe!("WskRegister", status);
            return Err(status);
        }
        Ok(())
    }
}

impl Drop for SocketRegistration {
    fn drop(&mut self) {
        unsafe { win::WskDeregister(&self.0) };
    }
}

pub struct UdpSocket(*const win::WSK_SOCKET);

impl UdpSocket {
    pub fn register() -> Result<(), NTSTATUS> {
        unsafe { REGISTRATION.init(|uninit| SocketRegistration::init(uninit)) }
    }

    pub fn deregister() {
        unsafe { REGISTRATION.drop() }
    }

    pub fn new(request: &mut IoRequest) -> Result<UdpSocketInitGuard, NTSTATUS> {
        let registration = unsafe { REGISTRATION.get() };
        let mut provider_npi = MaybeUninit::uninit();
        let status = unsafe {
            win::WskCaptureProviderNPI(
                &registration.0,
                win::WSK_INFINITE_WAIT,
                provider_npi.as_mut_ptr(),
            )
        };
        if !NT_SUCCESS(status) {
            trace_exit_status!("WskCaptureProviderNPI", status);
            return Err(status);
        }
        let result = {
            let provider_npi = unsafe { provider_npi.assume_init() };

            let dispatch = provider_npi.dispatch;
            let dispatch = unsafe { dispatch.as_ref().unwrap() };
            let wsk_socket = dispatch.wsk_socket.unwrap();
            let status = wsk_socket(
                provider_npi.client,
                AF_INET6,
                SOCK_DGRAM,
                IPPROTO::IPPROTO_UDP,
                win::WSK_FLAG_DATAGRAM_SOCKET,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                request.reuse()?,
            );
            let status = request.wait(status);
            if !NT_SUCCESS(status) {
                Err(status)
            } else {
                Ok(UdpSocketInitGuard {
                    socket: Self(request.info() as _),
                    request,
                })
            }
        };
        unsafe { win::WskReleaseProviderNPI(&registration.0) };
        result
    }

    fn basic_dispatch(&self) -> &win::WSK_PROVIDER_BASIC_DISPATCH {
        let socket = unsafe { &*self.0 };
        unsafe { &*socket.dispatch.cast() }
    }

    fn datagram_dispatch(&self) -> &win::WSK_PROVIDER_DATAGRAM_DISPATCH {
        let socket = unsafe { &*self.0 };
        unsafe { &*socket.dispatch.cast() }
    }

    pub fn set_option(
        &mut self,
        request: &mut IoRequest,
        option: u32,
        level: IPPROTO,
        value: bool,
    ) -> Result<(), NTSTATUS> {
        let value: u32 = if value { 1 } else { 0 };
        let dispatch = self.basic_dispatch();
        let wsk_control_socket = dispatch.wsk_control_socket.unwrap();
        let status = wsk_control_socket(
            self.0,
            win::WSK_CONTROL_SOCKET_TYPE::WskSetOption,
            option,
            level,
            mem::size_of_val(&value),
            value.as_ptr().cast(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            request.reuse()?,
        );
        let status = request.wait(status);
        if !NT_SUCCESS(status) {
            Err(status)
        } else {
            Ok(())
        }
    }

    pub fn bind(&mut self, request: &mut IoRequest, addr: &SOCKADDR_IN6) -> Result<(), NTSTATUS> {
        let dispatch = self.datagram_dispatch();
        let wsk_bind = dispatch.wsk_bind.unwrap();
        let status = wsk_bind(self.0, addr.as_ptr().cast(), 0, request.reuse()?);
        let status = request.wait(status);
        if !NT_SUCCESS(status) {
            Err(status)
        } else {
            Ok(())
        }
    }

    pub fn send_to(
        &self,
        request: &mut IoRequest,
        buf: &win::WSK_BUF,
        addr: &SOCKADDR_IN6,
    ) -> Result<usize, NTSTATUS> {
        let dispatch = self.datagram_dispatch();
        let wsk_send_to = dispatch.wsk_send_to.unwrap();
        let status = wsk_send_to(
            self.0,
            buf,
            0,
            addr.as_ptr().cast(),
            0,
            ptr::null(),
            request.reuse()?,
        );
        let status = request.wait(status);
        if !NT_SUCCESS(status) {
            trace_exit_status!("wsk_send_to", status);
            Err(status)
        } else {
            Ok(request.info())
        }
    }

    pub fn recv_from(
        &self,
        request: &mut IoRequest,
        buf: &win::WSK_BUF,
        addr: &mut MaybeUninit<SOCKADDR_IN6>,
    ) -> Result<usize, NTSTATUS> {
        let dispatch = self.datagram_dispatch();
        let wsk_receive_from = dispatch.wsk_receive_from.unwrap();
        let status = wsk_receive_from(
            self.0,
            buf,
            0,
            addr.as_mut_ptr().cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            request.reuse()?,
        );
        let status = request.wait(status);
        if !NT_SUCCESS(status) {
            trace_exit_status!("wsk_receive_from", status);
            Err(status)
        } else {
            Ok(request.info())
        }
    }

    pub fn close(&mut self, request: &mut IoRequest) -> Result<(), NTSTATUS> {
        let dispatch = self.basic_dispatch();
        let wsk_close_socket = dispatch.wsk_close_socket.unwrap();
        let status = wsk_close_socket(self.0, request.reuse()?);
        let status = request.wait(status);
        if !NT_SUCCESS(status) {
            Err(status)
        } else {
            Ok(())
        }
    }
}

pub struct UdpSocketInitGuard<'a> {
    socket: UdpSocket,
    request: &'a mut IoRequest,
}

impl UdpSocketInitGuard<'_> {
    pub fn get(&mut self) -> (&mut UdpSocket, &mut IoRequest) {
        (&mut self.socket, &mut self.request)
    }

    pub fn take(self) -> UdpSocket {
        let socket = unsafe { ptr::read(&self.socket) };
        core::mem::forget(self);
        socket
    }
}

impl Drop for UdpSocketInitGuard<'_> {
    fn drop(&mut self) {
        self.socket.close(self.request).unwrap()
    }
}

pub struct UdpSocketWorker<'a> {
    socket: &'a UdpSocket,
    pub request: &'a mut IoRequest, // TODO
}

impl<'a> UdpSocketWorker<'a> {
    pub fn new(socket: &'a UdpSocket, request: &'a mut IoRequest) -> Self {
        Self { socket, request }
    }

    pub fn send_to(&mut self, buf: &win::WSK_BUF, addr: &SOCKADDR_IN6) -> Result<usize, NTSTATUS> {
        self.socket.send_to(&mut self.request, buf, addr)
    }

    pub fn recv_from(
        &mut self,
        buf: &win::WSK_BUF,
        addr: &mut MaybeUninit<SOCKADDR_IN6>,
    ) -> Result<usize, NTSTATUS> {
        self.socket.recv_from(&mut self.request, buf, addr)
    }
}

pub struct IoRequest {
    irp: *mut IRP,
    event: AutoEvent,
    pending: bool,
    canceled: AtomicBool,
}

impl IoRequest {
    pub unsafe fn init(uninit: *mut Self) -> Result<InitGuard<Self>, NTSTATUS> {
        let irp = IoAllocateIrp(1, false);
        if irp.is_null() {
            Err(STATUS_INSUFFICIENT_RESOURCES)
        } else {
            ptr::raw_mut!((*uninit).irp).write(irp);
            AutoEvent::init(ptr::raw_mut!((*uninit).event));
            ptr::raw_mut!((*uninit).pending).write(false);
            ptr::raw_mut!((*uninit).canceled).write(AtomicBool::new(false));
            Ok(InitGuard::new(uninit))
        }
    }

    fn reuse(&mut self) -> Result<*mut IRP, NTSTATUS> {
        if self.canceled.load(Relaxed) {
            return Err(STATUS_CANCELLED);
        }

        assert!(!self.pending);
        unsafe { IoReuseIrp(self.irp, STATUS_SUCCESS) };
        unsafe {
            IoSetCompletionRoutine(
                self.irp,
                Some(Self::complete),
                self.as_mut_ptr().cast(),
                true,
                true,
                true,
            )
        };
        self.pending = true;
        Ok(self.irp)
    }

    fn wait(&mut self, status: NTSTATUS) -> NTSTATUS {
        assert!(self.pending);
        let result = if status != STATUS_PENDING {
            status
        } else {
            self.event.wait();
            let irp = unsafe { self.irp.as_ref().unwrap() };
            irp.io_status.status
        };
        self.pending = false;
        result
    }

    fn info(&mut self) -> usize {
        assert!(!self.pending);
        let irp = unsafe { self.irp.as_ref().unwrap() };
        irp.io_status.information
    }

    pub fn cancel(&self) {
        self.canceled.store(true, Relaxed);

        // CAUTION: It seems safe to cancel a completed IRP.
        unsafe { IoCancelIrp(self.irp) };
    }

    extern "system" fn complete(
        _device_object: *const DEVICE_OBJECT,
        _irp: *const IRP,
        context: PVOID,
    ) -> NTSTATUS {
        let request = unsafe { context.cast::<Self>().as_ref().unwrap() };
        request.event.set();

        STATUS_MORE_PROCESSING_REQUIRED
    }
}

impl Drop for IoRequest {
    fn drop(&mut self) {
        assert!(!self.pending);
        unsafe { IoFreeIrp(self.irp) }
    }
}
