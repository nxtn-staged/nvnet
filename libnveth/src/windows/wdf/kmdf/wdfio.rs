use core::{default::default, mem};

use libnveth_macros::*;

use crate::windows::{
    shared::ntdef::NTSTATUS,
    wdf::kmdf::{
        wdfobject::WDF_OBJECT_ATTRIBUTES,
        wdftypes::{WDFDEVICE, WDFQUEUE, WDFREQUEST, WDF_TRI_STATE},
    },
};

c_type!(
    pub enum WDF_IO_QUEUE_DISPATCH_TYPE {
        WdfIoQueueDispatchInvalid = 0,
        WdfIoQueueDispatchSequential,
        WdfIoQueueDispatchParallel,
    }
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_DEFAULT = fn() -> !;
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_STOP = fn() -> !;
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_RESUME = fn() -> !;
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_READ = fn() -> !;
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_WRITE = fn() -> !;
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_DEVICE_CONTROL = fn(
        queue: WDFQUEUE,
        request: WDFREQUEST,
        output_buffer_length: usize,
        input_buffer_length: usize,
        io_control_code: u32,
    ) -> ();
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_INTERNAL_DEVICE_CONTROL = fn() -> !;
);

c_type!(
    pub type PFN_WDF_IO_QUEUE_IO_CANCELED_ON_QUEUE = fn() -> !;
);

c_type!(
    pub struct WDF_IO_QUEUE_CONFIG {
        pub size: u32,
        pub dispatch_type: WDF_IO_QUEUE_DISPATCH_TYPE,
        pub power_managed: WDF_TRI_STATE,
        pub allow_zero_length_requests: bool,
        pub default_queue: bool,
        pub evt_io_default: PFN_WDF_IO_QUEUE_IO_DEFAULT,
        pub evt_io_read: PFN_WDF_IO_QUEUE_IO_READ,
        pub evt_io_write: PFN_WDF_IO_QUEUE_IO_WRITE,
        pub evt_io_device_control: PFN_WDF_IO_QUEUE_IO_DEVICE_CONTROL,
        pub evt_io_internal_device_control: PFN_WDF_IO_QUEUE_IO_INTERNAL_DEVICE_CONTROL,
        pub evt_io_stop: PFN_WDF_IO_QUEUE_IO_STOP,
        pub evt_io_resume: PFN_WDF_IO_QUEUE_IO_RESUME,
        pub evt_io_canceled_on_queue: PFN_WDF_IO_QUEUE_IO_CANCELED_ON_QUEUE,
        pub settings: WDF_IO_QUEUE_CONFIG_u,
    }
);

c_type!(
    pub union WDF_IO_QUEUE_CONFIG_u {
        pub parallel: WDF_IO_QUEUE_CONFIG_u_s,
    }
);

c_type!(
    pub struct WDF_IO_QUEUE_CONFIG_u_s {
        pub number_of_presented_requests: u32,
    }
);

pub fn WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(
    dispatch_type: WDF_IO_QUEUE_DISPATCH_TYPE,
) -> WDF_IO_QUEUE_CONFIG {
    let mut config = WDF_IO_QUEUE_CONFIG {
        size: mem::size_of::<WDF_IO_QUEUE_CONFIG>() as _,
        power_managed: WDF_TRI_STATE::WdfUseDefault,
        default_queue: true,
        dispatch_type,
        ..default()
    };
    if dispatch_type == WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchParallel {
        config.settings.parallel.number_of_presented_requests = -1 as _;
    }
    config
}

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfIoQueueCreate(
        device: WDFDEVICE,
        config: *const WDF_IO_QUEUE_CONFIG,
        queue_attributes: *const WDF_OBJECT_ATTRIBUTES,
        queue: *mut WDFQUEUE,
    ) -> NTSTATUS {
        WdfIoQueueCreateTableIndex
    }
);

wdf_fn!(
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn WdfIoQueueGetDevice(queue: WDFQUEUE) -> WDFDEVICE {
        WdfIoQueueGetDeviceTableIndex
    }
);
