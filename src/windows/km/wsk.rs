use core::ffi::c_void;

use crate::windows::{
    km::wdm::{EPROCESS, ETHREAD, IRP, KSPIN_LOCK, MDL, SECURITY_DESCRIPTOR},
    shared::{
        ntdef::{NTSTATUS, PVOID},
        ws2def::{ADDRESS_FAMILY, IPPROTO, SOCKADDR, WSACMSGHDR},
    },
};

c_type!(
    pub struct WSK_SOCKET {
        pub dispatch: *const c_void,
    }
);

c_type!(
    pub struct WSK_CLIENT;
);

pub const fn MAKE_WSK_VERSION(major: u16, minor: u16) -> u16 {
    (major << 8) | (minor & 0xff)
}

c_type!(
    pub struct WSK_BUF {
        pub mdl: *mut MDL,
        pub offset: u32,
        pub length: usize,
    }
);

c_type!(
    pub type PFN_WSK_CLIENT_EVENT = fn(
        client_context: PVOID,
        event_type: u32,
        information: *const c_void,
        information_length: usize,
    ) -> NTSTATUS;
);

pub const WSK_FLAG_DATAGRAM_SOCKET: u32 = 0x00000004;

c_type!(
    pub type PFN_WSK_SOCKET = fn(
        client: *mut WSK_CLIENT,
        address_family: ADDRESS_FAMILY,
        socket_type: u16,
        protocol: IPPROTO,
        flags: u32,
        socket_context: PVOID,
        dispatch: *const c_void,
        owning_process: *const EPROCESS,
        owning_thread: *const ETHREAD,
        security_descriptor: *const SECURITY_DESCRIPTOR,
        irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub type PFN_WSK_BIND = fn(
        socket: *const WSK_SOCKET,
        local_address: *const SOCKADDR,
        flags: u32,
        irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub enum WSK_CONTROL_SOCKET_TYPE {
        WskSetOption = 0,
    }
);

c_type!(
    pub type PFN_WSK_CONTROL_SOCKET = fn(
        socket: *const WSK_SOCKET,
        request_type: WSK_CONTROL_SOCKET_TYPE,
        control_code: u32,
        level: IPPROTO,
        input_size: usize,
        input_buffer: *const c_void,
        output_size: usize,
        output_buffer: *mut c_void,
        output_size_returned: *mut usize,
        irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub type PFN_WSK_CLOSE_SOCKET = fn(socket: *const WSK_SOCKET, irp: *mut IRP) -> NTSTATUS;
);

c_type!(
    pub type PFN_WSK_SEND_TO = fn(
        socket: *const WSK_SOCKET,
        buffer: *const WSK_BUF,
        flags: u32,
        remote_address: *const SOCKADDR,
        control_info_length: u32,
        control_info: *const WSACMSGHDR,
        irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub type PFN_WSK_RECEIVE_FROM = fn(
        socket: *const WSK_SOCKET,
        buffer: *const WSK_BUF,
        flags: u32,
        remote_address: *mut SOCKADDR,
        control_info_length: *mut u32, // control_length
        control_info: *mut WSACMSGHDR,
        control_flags: *mut u32,
        irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub struct WSK_CLIENT_DISPATCH {
        pub version: u16,
        pub reserved: u16,
        pub wsk_client_event: PFN_WSK_CLIENT_EVENT,
    }
);

c_type!(
    pub struct WSK_PROVIDER_DISPATCH {
        pub version: u16,
        pub reserved: u16,
        pub wsk_socket: PFN_WSK_SOCKET,
        // ...
    }
);

c_type!(
    pub struct WSK_PROVIDER_BASIC_DISPATCH {
        pub wsk_control_socket: PFN_WSK_CONTROL_SOCKET,
        pub wsk_close_socket: PFN_WSK_CLOSE_SOCKET,
    }
);

c_type!(
    pub struct WSK_PROVIDER_DATAGRAM_DISPATCH {
        pub basic: WSK_PROVIDER_BASIC_DISPATCH,
        pub wsk_bind: PFN_WSK_BIND,
        pub wsk_send_to: PFN_WSK_SEND_TO,
        pub wsk_receive_from: PFN_WSK_RECEIVE_FROM,
        // ...
    }
);

c_type!(
    pub struct WSK_CLIENT_NPI {
        pub client_context: PVOID,
        pub dispatch: *const WSK_CLIENT_DISPATCH,
    }
);

c_type!(
    pub struct WSK_PROVIDER_NPI {
        pub client: *mut WSK_CLIENT,
        pub dispatch: *const WSK_PROVIDER_DISPATCH,
    }
);

c_type!(
    pub struct WSK_REGISTRATION {
        pub reserved_registration_state: u64,
        pub reserved_registration_context: PVOID,
        pub reserved_registration_lock: KSPIN_LOCK,
    }
);

extern "system" {
    pub fn WskRegister(
        wsk_client_npi: *const WSK_CLIENT_NPI,
        wsk_registration: *mut WSK_REGISTRATION,
    ) -> NTSTATUS;
}

pub const WSK_INFINITE_WAIT: u32 = 0xffffffff;

extern "system" {
    pub fn WskCaptureProviderNPI(
        wsk_registration: *const WSK_REGISTRATION,
        wait_timeout: u32,
        wsk_provider_npi: *mut WSK_PROVIDER_NPI,
    ) -> NTSTATUS;

    pub fn WskReleaseProviderNPI(wsk_registration: *const WSK_REGISTRATION) -> ();

    pub fn WskDeregister(wsk_registration: *const WSK_REGISTRATION) -> ();
}
