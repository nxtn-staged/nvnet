use crate::windows::shared::{
    ndis::{status::NDIS_STATUS_SUCCESS, types::NDIS_STATUS},
    ntdef::NTSTATUS,
    ntstatus::STATUS_SUCCESS,
};

pub type Result<T> = core::result::Result<T, NTSTATUS>;

impl From<Result<()>> for NDIS_STATUS {
    fn from(result: Result<()>) -> Self {
        match result {
            Ok(()) => NDIS_STATUS_SUCCESS,
            Err(status) => status.into(),
        }
    }
}

impl From<Result<()>> for NTSTATUS {
    fn from(result: Result<()>) -> Self {
        match result {
            Ok(()) => STATUS_SUCCESS,
            Err(status) => status,
        }
    }
}
