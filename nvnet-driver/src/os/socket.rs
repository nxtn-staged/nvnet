use core::{cell::UnsafeCell, default::default, ffi::c_void, mem, ptr};

use crate::{
    debug::ResultExt,
    os::request::SyncRequest,
    windows::{
        km::{
            wdm::{
                IoInitializeIrp, IoReuseIrp, IoSetCompletionRoutine, DEVICE_OBJECT,
                IO_STACK_LOCATION, IRP,
            },
            wsk::{
                WskCaptureProviderNPI, WskDeregister, WskRegister, WskReleaseProviderNPI,
                MAKE_WSK_VERSION, NPI_WSK_INTERFACE_ID, WSK_BUF_LIST, WSK_CLIENT_DATAGRAM_DISPATCH,
                WSK_CLIENT_DISPATCH, WSK_CLIENT_NPI, WSK_CONTROL_SOCKET_TYPE,
                WSK_DATAGRAM_INDICATION, WSK_EVENT_CALLBACK_CONTROL, WSK_EVENT_RECEIVE_FROM,
                WSK_FLAG_DATAGRAM_SOCKET, WSK_INFINITE_WAIT, WSK_PROVIDER_BASIC_DISPATCH,
                WSK_PROVIDER_DATAGRAM_DISPATCH, WSK_PROVIDER_NPI, WSK_REGISTRATION,
                WSK_SET_STATIC_EVENT_CALLBACKS, WSK_SOCKET,
            },
        },
        shared::{
            ntdef::NTSTATUS,
            ntstatus::STATUS_SUCCESS,
            ws2def::{AF_INET6, IPPROTO, SOCK_DGRAM},
            ws2ipdef::{IPV6_V6ONLY, SOCKADDR_IN6},
        },
        OkExt, Result,
    },
};

pub struct SocketClient {
    registration: WSK_REGISTRATION,
    provider_npi: WSK_PROVIDER_NPI,
    client_dispatch: WSK_CLIENT_DATAGRAM_DISPATCH,
}

impl SocketClient {
    pub unsafe fn init<'a>(
        uninit: *mut Self,
        evt_recv_from: extern "system" fn(
            *mut c_void,
            u32,
            *mut WSK_DATAGRAM_INDICATION,
        ) -> NTSTATUS,
    ) -> Result<&'a mut Self> {
        let mut client_dispatch = WSK_CLIENT_DISPATCH {
            Version: MAKE_WSK_VERSION(1, 0),
            ..default()
        };
        let mut client_npi = WSK_CLIENT_NPI {
            Dispatch: &mut client_dispatch,
            ..default()
        };
        WskRegister(&mut client_npi, ptr::addr_of_mut!((*uninit).registration))
            .ok()
            .context_exit("WskRegister")?;
        let result = (|| {
            WskCaptureProviderNPI(
                &mut (*uninit).registration,
                WSK_INFINITE_WAIT,
                ptr::addr_of_mut!((*uninit).provider_npi),
            )
            .ok()
            .context_exit("WskCaptureProviderNPI")?;
            let result = (|| {
                let mut event_callback_control = WSK_EVENT_CALLBACK_CONTROL {
                    NpiId: unsafe { &NPI_WSK_INTERFACE_ID },
                    EventMask: WSK_EVENT_RECEIVE_FROM,
                };
                let dispatch = &*(*uninit).provider_npi.Dispatch;
                let wsk_control_client = unsafe { dispatch.WskControlClient.unwrap_unchecked() };
                wsk_control_client(
                    (*uninit).provider_npi.Client,
                    WSK_SET_STATIC_EVENT_CALLBACKS,
                    mem::size_of_val(&event_callback_control),
                    &mut event_callback_control as *mut _ as *mut _,
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
                .ok()
                .context_exit("WskControlClient")?;
                ptr::addr_of_mut!((*uninit).client_dispatch).write(WSK_CLIENT_DATAGRAM_DISPATCH {
                    WskReceiveFromEvent: Some(evt_recv_from),
                });
                Ok(&mut *uninit)
            })();
            if result.is_err() {
                WskReleaseProviderNPI(&mut (*uninit).registration);
            }
            result
        })();
        if result.is_err() {
            WskDeregister(&mut (*uninit).registration);
        }
        result
    }
}

impl Drop for SocketClient {
    fn drop(&mut self) {
        unsafe { WskReleaseProviderNPI(&mut self.registration) };
        unsafe { WskDeregister(&mut self.registration) };
    }
}

pub struct UdpSocket(*mut WSK_SOCKET);

impl UdpSocket {
    pub fn new(client: &SocketClient, request: &SyncRequest, context: *mut c_void) -> Result<Self> {
        let provider_npi = &client.provider_npi;
        let dispatch = unsafe { &*provider_npi.Dispatch };
        let wsk_socket = unsafe { dispatch.WskSocket.unwrap_unchecked() };
        request
            .invoke(|irp| {
                wsk_socket(
                    provider_npi.Client,
                    AF_INET6,
                    SOCK_DGRAM,
                    IPPROTO::IPPROTO_UDP,
                    WSK_FLAG_DATAGRAM_SOCKET,
                    context as *const _ as *mut _,
                    &client.client_dispatch as *const _ as *const _,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    irp,
                )
            })
            .context_exit("WskSocket")?;
        Ok(Self(request.info() as *mut _))
    }

    fn basic_dispatch(&self) -> &WSK_PROVIDER_BASIC_DISPATCH {
        let socket = unsafe { &*self.0 };
        unsafe { &*socket.Dispatch.cast() }
    }

    fn datagram_dispatch(&self) -> &WSK_PROVIDER_DATAGRAM_DISPATCH {
        let socket = unsafe { &*self.0 };
        unsafe { &*socket.Dispatch.cast() }
    }

    fn set_option(
        &self,
        request: &SyncRequest,
        option: u32,
        level: IPPROTO,
        value: bool,
    ) -> Result<()> {
        let mut value: u32 = value.into();
        let dispatch = self.basic_dispatch();
        let wsk_control_socket = unsafe { dispatch.WskControlSocket.unwrap_unchecked() };
        request
            .invoke(|irp| {
                wsk_control_socket(
                    self.0,
                    WSK_CONTROL_SOCKET_TYPE::WskSetOption,
                    option,
                    level,
                    mem::size_of_val(&value),
                    &mut value as *mut _ as *mut _,
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    irp,
                )
            })
            .context_exit("WskControlSocket")?;
        Ok(())
    }

    pub fn set_ipv6_only(&self, request: &SyncRequest, ipv6_only: bool) -> Result<()> {
        self.set_option(request, IPV6_V6ONLY, IPPROTO::IPPROTO_IPV6, ipv6_only)?;
        Ok(())
    }

    pub fn bind(&self, request: &SyncRequest, addr: &SOCKADDR_IN6) -> Result<()> {
        let dispatch = self.datagram_dispatch();
        let wsk_bind = unsafe { dispatch.WskBind.unwrap_unchecked() };
        request
            .invoke(|irp| wsk_bind(self.0, addr as *const _ as *mut _, 0, irp))
            .context_exit("WskBind")?;
        Ok(())
    }

    /*
    pub fn send_to(
        &self,
        request: &SyncRequest,
        buf: &mut WSK_BUF,
        addr: &SOCKADDR_IN6,
    ) -> Result<usize> {
        let dispatch = self.datagram_dispatch();
        let wsk_send_to = unsafe { dispatch.WskSendTo.unwrap_unchecked() };
        request
            .invoke(|irp| {
                wsk_send_to(
                    self.0,
                    buf,
                    0,
                    addr as *const _ as *mut _,
                    0,
                    ptr::null_mut(),
                    irp,
                )
            })
            .context_exit("WskSendTo")?;
        Ok(request.info())
    }
    */

    pub fn send_messages(
        &self,
        request: &SyncRequest,
        buf_list: &mut WSK_BUF_LIST,
        addr: &SOCKADDR_IN6,
    ) -> Result<usize> {
        let dispatch = self.datagram_dispatch();
        let wsk_send_messages = unsafe { dispatch.WskSendMessages.unwrap_unchecked() };
        request
            .invoke(|irp| {
                wsk_send_messages(
                    self.0,
                    buf_list,
                    0,
                    addr as *const _ as *mut _,
                    0,
                    ptr::null_mut(),
                    irp,
                )
            })
            .context_exit("WskSendMessages")?;
        Ok(request.info())
    }

    pub fn send_messages_async(
        &self,
        request: &SocketRequest,
        buf_list: &mut WSK_BUF_LIST,
        addr: &SOCKADDR_IN6,
        complete: extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, *mut c_void) -> NTSTATUS,
        complete_context: *mut c_void,
    ) -> Result<()> {
        let dispatch = self.datagram_dispatch();
        let wsk_send_messages = unsafe { dispatch.WskSendMessages.unwrap_unchecked() };
        request
            .invoke_async(complete, complete_context, |irp| {
                wsk_send_messages(
                    self.0,
                    buf_list,
                    0,
                    addr as *const _ as *mut _,
                    0,
                    ptr::null_mut(),
                    irp,
                )
            })
            .context_exit("WskSendMessages")?;
        Ok(())
    }

    pub fn release(&self, datagram_indication: *mut WSK_DATAGRAM_INDICATION) -> Result<()> {
        let dispatch = self.datagram_dispatch();
        let wsk_release = unsafe { dispatch.WskRelease.unwrap_unchecked() };
        wsk_release(self.0, datagram_indication)
            .ok()
            .context_exit("WskRelease")?;
        Ok(())
    }

    pub fn close(&self, request: &SyncRequest) -> Result<()> {
        let dispatch = self.basic_dispatch();
        let wsk_close_socket = unsafe { (*dispatch).WskCloseSocket.unwrap_unchecked() };
        request
            .invoke(|irp| wsk_close_socket(self.0, irp))
            .context_exit("WskCloseSocket")?;
        Ok(())
    }
}

pub struct SocketRequest(UnsafeCell<IrpRepr<1>>);

impl SocketRequest {
    pub unsafe fn init<'a>(uninit: *mut Self) -> &'a mut Self {
        IrpRepr::init(UnsafeCell::raw_get(ptr::addr_of!((*uninit).0)));
        &mut *uninit
    }

    pub fn invoke_async(
        &self,
        complete: extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, *mut c_void) -> NTSTATUS,
        complete_context: *mut c_void,
        invoke: impl FnOnce(*mut IRP) -> NTSTATUS,
    ) -> Result<()> {
        let irp = unsafe { self.0.get().raw_get() };
        unsafe { IoReuseIrp(irp, STATUS_SUCCESS) };
        unsafe { IoSetCompletionRoutine(irp, Some(complete), complete_context, true, true, true) };
        invoke(irp).ok()
    }

    pub fn info(&self) -> Result<usize> {
        let irp = unsafe { self.0.get().raw_get() };
        unsafe {
            (*irp)
                .IoStatus
                .Status
                .ok()
                .map(|()| (*irp).IoStatus.Information)
        }
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
