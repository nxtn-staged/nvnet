#![no_std]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(const_size_of_val)]
#![feature(core_intrinsics)]
#![feature(default_alloc_error_handler)]
#![feature(default_free_fn)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_ref)]
#![feature(option_unwrap_none)]
#![feature(raw_ref_macros)]
#![feature(try_reserve)]
#![feature(untagged_unions)]

#[macro_use]
mod debug;

#[macro_use]
mod macros;

mod adapter;
mod allocator;
mod crypto;
mod device;
mod driver;
mod ext;
mod init;
mod ioctl;
mod list;
mod net;
mod os;
mod panic;
mod peer;
mod recv;
mod send;
mod socket;
mod windows;
mod worker;

extern crate alloc;

use core::ptr;

use sal::*;

use crate::{
    socket::UdpSocket,
    windows::{
        prelude as win,
        shared::{
            ntdef::{NTSTATUS, NT_SUCCESS, UNICODE_STRING},
            ntstatus::STATUS_SUCCESS,
        },
    },
};

const MAX_FRAME_DATA_SIZE: u32 = {
    const MAX_ETH_MTU_SIZE: u32 = 1500;
    const MAX_IP_HEADER_SIZE: u32 = 60;
    const MAX_UDP_HEADER_SIZE: u32 = 8;
    const MAX_ETH_HEADER_SIZE: u32 = 14;
    MAX_ETH_MTU_SIZE - MAX_IP_HEADER_SIZE - MAX_UDP_HEADER_SIZE - MAX_ETH_HEADER_SIZE
}; // 1418

const LINK_SPEED: u64 = 10_000_000_000; // 10.0 Gbps

#[no_mangle]
#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn DriverEntry(
    driver_object: *const win::DRIVER_OBJECT,
    registry_path: *const UNICODE_STRING,
) -> NTSTATUS {
    trace_entry!("DriverEntry");

    let status = (|| {
        let mut driver_config = win::WDF_DRIVER_CONFIG_INIT(Some(driver::evt_driver_device_add));
        driver_config.evt_driver_unload = Some(driver::evt_driver_device_unload);
        let status = unsafe {
            win::WdfDriverCreate(
                driver_object,
                registry_path,
                ptr::null(),
                &driver_config,
                ptr::null_mut(),
            )
        };
        if !NT_SUCCESS(status) {
            trace_exit_status!("WdfDriverCreate", status);
            return status;
        }
        if let Err(status) = UdpSocket::register() {
            return status;
        }
        STATUS_SUCCESS
    })();

    trace_exit_status!("DriverEntry", status);
    status
}
