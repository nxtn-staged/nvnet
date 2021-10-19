use core::ffi::c_void;

use crate::windows::shared::ndis::{
    ndisport::NDIS_PORT_NUMBER,
    objectheader::NDIS_OBJECT_HEADER,
    oidtypes::{NDIS_OID, NDIS_REQUEST_TYPE},
    types::NDIS_HANDLE,
};

// L23
c_type!(
    pub struct NDIS_OID_REQUEST {
        pub Header: NDIS_OBJECT_HEADER,
        pub RequestType: NDIS_REQUEST_TYPE,
        pub PortNumber: NDIS_PORT_NUMBER,
        pub Timeout: u32,
        pub RequestId: *mut c_void,
        pub RequestHandle: NDIS_HANDLE,
        pub DATA: NDIS_OID_REQUEST_0,
        // ...
    }
);

c_type!(
    pub union NDIS_OID_REQUEST_0 {
        pub QUERY_INFORMATION: NDIS_OID_REQUEST_0_0,
        pub SET_INFORMATION: NDIS_OID_REQUEST_0_1,
        // ...
    }
);

c_type!(
    pub struct NDIS_OID_REQUEST_0_0 {
        pub Oid: NDIS_OID,
        pub InformationBuffer: *mut c_void,
        pub InformationBufferLength: u32,
        pub BytesWritten: u32,
        pub BytesNeeded: u32,
    }
);

c_type!(
    pub struct NDIS_OID_REQUEST_0_1 {
        pub Oid: NDIS_OID,
        pub InformationBuffer: *mut c_void,
        pub InformationBufferLength: u32,
        pub BytesRead: u32,
        pub BytesNeeded: u32,
    }
);
