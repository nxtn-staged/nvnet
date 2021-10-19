use core::ffi::c_void;

use crate::windows::{
    km::wdm::{EPROCESS, ETHREAD, IRP, KSPIN_LOCK, MDL, SECURITY_DESCRIPTOR},
    shared::{
        netiodef::NPIID,
        ntdef::NTSTATUS,
        ws2def::{ADDRESS_FAMILY, IPPROTO, SOCKADDR, WSACMSGHDR},
    },
};

// L42
c_type!(
    pub struct WSK_SOCKET {
        pub Dispatch: *const c_void,
    }
);

c_type!(
    pub struct WSK_CLIENT;
);

extern "system" {
    pub static NPI_WSK_INTERFACE_ID: NPIID;
}

pub const fn MAKE_WSK_VERSION(major: u16, minor: u16) -> u16 {
    (major << 8) | (minor & 0xff)
}

// L106
c_type!(
    pub struct WSK_BUF {
        pub Mdl: *mut MDL,
        pub Offset: u32,
        pub Length: usize,
    }
);

// L127
c_type!(
    pub struct WSK_DATAGRAM_INDICATION {
        pub Next: *mut WSK_DATAGRAM_INDICATION,
        pub Buffer: WSK_BUF,
        pub ControlInfo: *mut WSACMSGHDR,
        pub ControlInfoLength: u32,
        pub RemoteAddress: *mut SOCKADDR,
    }
);

// L161
pub const WSK_FLAG_AT_DISPATCH_LEVEL: u32 = 0x00000008;

c_type!(
    pub type PFN_WSK_CLIENT_EVENT = fn() -> !;
);

c_type!(
    pub type PFN_WSK_RECEIVE_FROM_EVENT = fn(
        SocketContext: *mut c_void,
        Flags: u32,
        DataIndication: *mut WSK_DATAGRAM_INDICATION,
    ) -> NTSTATUS;
);

// L462
pub const WSK_FLAG_DATAGRAM_SOCKET: u32 = 0x00000004;

c_type!(
    pub type PFN_WSK_SOCKET = fn(
        Client: *mut WSK_CLIENT,
        AddressFamily: ADDRESS_FAMILY,
        SocketType: u16,
        Protocol: IPPROTO,
        Flags: u32,
        SocketContext: *mut c_void,
        Dispatch: *const c_void,
        OwningProcess: *mut EPROCESS,
        OwningThread: *mut ETHREAD,
        SecurityDescriptor: *mut SECURITY_DESCRIPTOR,
        Irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub type PFN_WSK_SOCKET_CONNECT = fn() -> !;
);

pub const WSK_SET_STATIC_EVENT_CALLBACKS: u32 = 7;

c_type!(
    pub type PFN_WSK_CONTROL_CLIENT = fn(
        Client: *mut WSK_CLIENT,
        ControlCode: u32,
        InputSize: usize,
        InputBuffer: *mut c_void,
        OutputSize: usize,
        OutputBuffer: *mut c_void,
        OutputSizeReturned: *mut usize,
        Irp: *mut IRP,
    ) -> NTSTATUS;
);

// L792
c_type!(
    pub type PFN_WSK_BIND = fn(
        Socket: *mut WSK_SOCKET,
        LocalAddress: *mut SOCKADDR,
        Flags: u32,
        Irp: *mut IRP,
    ) -> NTSTATUS;
);

// L882
pub const WSK_EVENT_RECEIVE_FROM: u32 = 0x00000100;

c_type!(
    pub struct WSK_EVENT_CALLBACK_CONTROL {
        pub NpiId: *const NPIID,
        pub EventMask: u32,
    }
);

// L940
c_type!(
    pub enum WSK_CONTROL_SOCKET_TYPE {
        WskSetOption = 0,
    }
);

c_type!(
    pub type PFN_WSK_CONTROL_SOCKET = fn(
        Socket: *mut WSK_SOCKET,
        RequestType: WSK_CONTROL_SOCKET_TYPE,
        ControlCode: u32,
        Level: IPPROTO,
        InputSize: usize,
        InputBuffer: *mut c_void,
        OutputSize: usize,
        OutputBuffer: *mut c_void,
        OutputSizeReturned: *mut usize,
        Irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub type PFN_WSK_CLOSE_SOCKET = fn(Socket: *mut WSK_SOCKET, Irp: *mut IRP) -> NTSTATUS;
);

// L1091
c_type!(
    pub type PFN_WSK_SEND_TO = fn(
        Socket: *mut WSK_SOCKET,
        Buffer: *mut WSK_BUF,
        Flags: u32,
        RemoteAddress: *mut SOCKADDR,
        ControlInfoLength: u32,
        ControlInfo: *mut WSACMSGHDR,
        Irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub struct WSK_BUF_LIST {
        pub Next: *mut WSK_BUF_LIST,
        pub Buffer: WSK_BUF,
    }
);

c_type!(
    pub type PFN_WSK_SEND_MESSAGES = fn(
        Socket: *mut WSK_SOCKET,
        BufferList: *mut WSK_BUF_LIST,
        Flags: u32,
        RemoteAddress: *mut SOCKADDR,
        ControlInfoLength: u32,
        ControlInfo: *mut WSACMSGHDR,
        Irp: *mut IRP,
    ) -> NTSTATUS;
);

c_type!(
    pub type PFN_WSK_RECEIVE_FROM = fn() -> !;
);

c_type!(
    pub type PFN_WSK_GET_LOCAL_ADDRESS = fn() -> !;
);

c_type!(
    pub type PFN_WSK_RELEASE_DATAGRAM_INDICATION_LIST =
        fn(Socket: *mut WSK_SOCKET, DatagramIndication: *mut WSK_DATAGRAM_INDICATION) -> NTSTATUS;
);

// L1485
c_type!(
    pub struct WSK_CLIENT_DISPATCH {
        pub Version: u16,
        pub Reserved: u16,
        pub WskClientEvent: PFN_WSK_CLIENT_EVENT,
    }
);

c_type!(
    pub struct WSK_CLIENT_DATAGRAM_DISPATCH {
        pub WskReceiveFromEvent: PFN_WSK_RECEIVE_FROM_EVENT,
    }
);

// L1534
c_type!(
    pub struct WSK_PROVIDER_DISPATCH {
        pub Version: u16,
        pub Reserved: u16,
        pub WskSocket: PFN_WSK_SOCKET,
        pub WskSocketConnect: PFN_WSK_SOCKET_CONNECT,
        pub WskControlClient: PFN_WSK_CONTROL_CLIENT,
        // ...
    }
);

c_type!(
    pub struct WSK_PROVIDER_BASIC_DISPATCH {
        pub WskControlSocket: PFN_WSK_CONTROL_SOCKET,
        pub WskCloseSocket: PFN_WSK_CLOSE_SOCKET,
    }
);

c_type!(
    pub struct WSK_PROVIDER_DATAGRAM_DISPATCH {
        pub Basic: WSK_PROVIDER_BASIC_DISPATCH,
        pub WskBind: PFN_WSK_BIND,
        pub WskSendTo: PFN_WSK_SEND_TO,
        pub WskReceiveFrom: PFN_WSK_RECEIVE_FROM,
        pub WskRelease: PFN_WSK_RELEASE_DATAGRAM_INDICATION_LIST,
        pub WskGetLocalAddress: PFN_WSK_GET_LOCAL_ADDRESS,
        pub WskSendMessages: PFN_WSK_SEND_MESSAGES,
    }
);

// L1651
c_type!(
    pub struct WSK_CLIENT_NPI {
        pub ClientContext: *mut c_void,
        pub Dispatch: *const WSK_CLIENT_DISPATCH,
    }
);

c_type!(
    pub struct WSK_PROVIDER_NPI {
        pub Client: *mut WSK_CLIENT,
        pub Dispatch: *const WSK_PROVIDER_DISPATCH,
    }
);

c_type!(
    pub struct WSK_REGISTRATION {
        ReservedRegistrationState: u64,
        ReservedRegistrationContext: *mut c_void,
        ReservedRegistrationLock: KSPIN_LOCK,
    }
);

extern "system" {
    // #[irql_requires(PASSIVE_LEVEL)]
    pub fn WskRegister(
        WskClientNpi: *mut WSK_CLIENT_NPI,
        WskRegistration: *mut WSK_REGISTRATION,
    ) -> NTSTATUS;
}

pub const WSK_INFINITE_WAIT: u32 = 0xffffffff;

extern "system" {
    // #[irql_requires(PASSIVE_LEVEL)]
    // ...
    pub fn WskCaptureProviderNPI(
        WskRegistration: *mut WSK_REGISTRATION,
        WaitTimeout: u32,
        WskProviderNpi: *mut WSK_PROVIDER_NPI,
    ) -> NTSTATUS;

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WskReleaseProviderNPI(WskRegistration: *mut WSK_REGISTRATION);

    // #[irql_requires(PASSIVE_LEVEL)]
    pub fn WskDeregister(WskRegistration: *mut WSK_REGISTRATION);
}
