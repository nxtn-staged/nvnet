use alloc::vec::Vec;

use core::{
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr,
};

use libnveth_macros::*;

use crate::{
    crypto::aes_gcm::AesGcm,
    list::BufPool,
    net::MacAddr,
    peer::Peer,
    recv::{self, VEthRxQueue},
    send::{self, VEthTxQueue},
    socket::{IoRequest, UdpSocket},
    windows::{
        km::ntifs::RtlRandomEx,
        prelude as win,
        shared::ifdef::{NET_IF_MEDIA_CONNECT_STATE, NET_IF_MEDIA_DUPLEX_STATE},
    },
};

pub struct VEthAdapter {
    init: bool, // zero-init

    pub adapter_handle: win::NETADAPTER, // pre-init

    pub local_mac_addr: MacAddr,

    peers: Vec<Peer>, // TODO

    pub tx_request: IoRequest,
    pub rx_request: IoRequest,

    pub socket: UdpSocket,
    request: IoRequest,

    pub rx_buf_pool: BufPool<VEthCipherFrame>,
}

impl VEthAdapter {
    pub fn pre_init(adapter: win::NETADAPTER) -> *mut Self {
        unsafe {
            let uninit = Self::from_adapter_mut_ptr(adapter);

            ptr::raw_mut!((*uninit).adapter_handle).write(adapter);

            uninit
        }
    }

    pub fn init<'a>(device: win::WDFDEVICE) -> Result<&'a mut Self, win::NTSTATUS> {
        unsafe {
            let uninit = Self::from_device_mut_ptr(device);

            let tx_request = IoRequest::init(ptr::raw_mut!((*uninit).tx_request))?;
            let rx_request = IoRequest::init(ptr::raw_mut!((*uninit).rx_request))?;
            let request = IoRequest::init(ptr::raw_mut!((*uninit).request))?;

            let mut socket_init = UdpSocket::new(&mut (*uninit).tx_request)?;
            let (socket, context) = socket_init.get();
            socket.set_option(context, win::IPV6_V6ONLY, win::IPPROTO::IPPROTO_IPV6, false)?;

            let local_addr = win::SOCKADDR_IN6 {
                family: win::AF_INET6,
                port: 5001u16.to_be(),
                addr: [0; 16],
                ..core::default::default()
            };
            socket.bind(context, &local_addr)?; // TODO remove

            // const TX_POOL_TAG: u32 = u32::from_ne_bytes([b'N', b'V', b'E', b'T']);
            const RX_POOL_TAG: u32 = u32::from_ne_bytes([b'N', b'V', b'E', b'R']);

            let rx_buf_pool = BufPool::init(ptr::raw_mut!((*uninit).rx_buf_pool), RX_POOL_TAG)?;

            let local_mac_addr = {
                let mut seed = 42u32;
                let random = RtlRandomEx(&mut seed);
                let random = random.to_ne_bytes();
                MacAddr::new(0x02, 0x00, random[0], random[1], random[2], random[3])
            };
            ptr::raw_mut!((*uninit).local_mac_addr).write(local_mac_addr);

            ptr::raw_mut!((*uninit).peers).write(Vec::new());

            mem::forget(tx_request);
            mem::forget(rx_request);

            ptr::raw_mut!((*uninit).socket).write(socket_init.take());
            mem::forget(request);

            mem::forget(rx_buf_pool);

            let init = &mut *uninit;
            init.init = true;
            Ok(init)
        }
    }

    pub fn from_adapter_mut<'a>(adapter: win::NETADAPTER) -> &'a mut Self {
        unsafe { &mut *Self::from_adapter_mut_ptr(adapter) }
    }

    fn from_adapter_mut_ptr(adapter: win::NETADAPTER) -> *mut Self {
        veth_get_adapter_context(adapter.into())
    }

    pub fn from_device_mut<'a>(device: win::WDFDEVICE) -> &'a mut Self {
        VEthAdapterPtr::from_device_mut(device)
    }

    fn from_device_mut_ptr(device: win::WDFDEVICE) -> *mut Self {
        VEthAdapterPtr::from_device_mut(device).0
    }

    pub fn set_connect_state(&mut self, connected: bool) {
        let link_state = win::NET_ADAPTER_LINK_STATE_INIT(
            crate::LINK_SPEED,
            if connected {
                NET_IF_MEDIA_CONNECT_STATE::MediaConnectStateConnected
            } else {
                NET_IF_MEDIA_CONNECT_STATE::MediaConnectStateDisconnected
            },
            NET_IF_MEDIA_DUPLEX_STATE::MediaDuplexStateFull,
            win::NET_ADAPTER_PAUSE_FUNCTION_TYPE::NetAdapterPauseFunctionTypeUnsupported,
            win::NET_ADAPTER_AUTO_NEGOTIATION_FLAGS::NetAdapterAutoNegotiationFlagNone,
        );
        unsafe { win::NetAdapterSetLinkState(self.adapter_handle, &link_state) };
    }

    pub fn set_local_addr(&mut self, local_addr: win::SOCKADDR_IN6) -> Result<(), win::NTSTATUS> {
        // self.socket.bind(&mut self.request, &local_addr)?;
        // self.socket.bind(&mut self.rx_request, &local_addr)?;
        Ok(())
    }

    pub fn add_peer(&mut self, remote_addr: win::SOCKADDR_IN6) -> Result<(), win::NTSTATUS> {
        if self.peers.try_reserve(1).is_err() {
            return Err(win::STATUS_INSUFFICIENT_RESOURCES);
        }
        self.peers.push(Peer::new(remote_addr));
        Ok(())
    }

    fn init_tx_queue(
        &'static mut self,
        tx_queue: win::NETPACKETQUEUE,
    ) -> Result<&mut VEthTxQueue, win::NTSTATUS> {
        VEthTxQueue::init(tx_queue, &self.socket, &mut self.tx_request, &self.peers)
    }

    fn init_rx_queue(
        &'static mut self,
        rx_queue: win::NETPACKETQUEUE,
    ) -> Result<&mut VEthRxQueue, win::NTSTATUS> {
        VEthRxQueue::init(rx_queue, &self.socket, &mut self.rx_request, &self.peers)
    }

    pub fn drop(&mut self) {
        if self.init {
            unsafe { ptr::drop_in_place(self) }
        }
    }
}

impl Drop for VEthAdapter {
    fn drop(&mut self) {
        self.socket.close(&mut self.tx_request).unwrap();
    }
}

// WDF_DECLARE_CONTEXT_TYPE_WITH_NAME
static mut WDF_VETH_ADAPTER_TYPE_INFO: win::WDF_OBJECT_CONTEXT_TYPE_INFO =
    win::WDF_OBJECT_CONTEXT_TYPE_INFO {
        size: mem::size_of::<win::WDF_OBJECT_CONTEXT_TYPE_INFO>() as _,
        context_name: "VEthAdapter\0".as_ptr(),
        context_size: mem::size_of::<VEthAdapter>(),
        unique_type: unsafe { &WDF_VETH_ADAPTER_TYPE_INFO },
        evt_driver_get_unique_context_type: None,
    };

fn veth_get_adapter_context(handle: win::WDFOBJECT) -> *mut VEthAdapter {
    unsafe {
        win::WdfObjectGetTypedContextWorker(handle, WDF_VETH_ADAPTER_TYPE_INFO.unique_type).cast()
    }
}

pub fn veth_get_adapter_context_type() -> *const win::WDF_OBJECT_CONTEXT_TYPE_INFO {
    unsafe { &WDF_VETH_ADAPTER_TYPE_INFO }
}

pub struct VEthAdapterPtr(*mut VEthAdapter);

impl VEthAdapterPtr {
    pub fn init(device: win::WDFDEVICE, adapter: *mut VEthAdapter) {
        let uninit = Self::from_device_mut(device);
        uninit.0 = adapter
    }

    fn from_device_mut<'a>(device: win::WDFDEVICE) -> &'a mut Self {
        unsafe { core::intrinsics::assert_zero_valid::<Self>() };
        unsafe { &mut *veth_get_adapter_ptr_context(device.into()) }
    }
}

impl Deref for VEthAdapterPtr {
    type Target = VEthAdapter;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for VEthAdapterPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

// WDF_DECLARE_CONTEXT_TYPE_WITH_NAME
static mut WDF_VETH_ADAPTER_PTR_TYPE_INFO: win::WDF_OBJECT_CONTEXT_TYPE_INFO =
    win::WDF_OBJECT_CONTEXT_TYPE_INFO {
        size: mem::size_of::<win::WDF_OBJECT_CONTEXT_TYPE_INFO>() as _,
        context_name: "VEthAdapterPtr\0".as_ptr(),
        context_size: mem::size_of::<VEthAdapterPtr>(),
        unique_type: unsafe { &WDF_VETH_ADAPTER_PTR_TYPE_INFO },
        evt_driver_get_unique_context_type: None,
    };

fn veth_get_adapter_ptr_context(handle: win::WDFOBJECT) -> *mut VEthAdapterPtr {
    unsafe {
        win::WdfObjectGetTypedContextWorker(handle, WDF_VETH_ADAPTER_PTR_TYPE_INFO.unique_type)
            .cast()
    }
}

pub fn veth_get_adapter_ptr_context_type() -> *const win::WDF_OBJECT_CONTEXT_TYPE_INFO {
    unsafe { &WDF_VETH_ADAPTER_PTR_TYPE_INFO }
}

#[repr(C)]
pub struct VEthPlainFrame {
    pub data: [u8; crate::PLAIN_FRAME_DATA_SIZE as _],
}

#[repr(C)]
pub struct VEthCipherFrameHeader {
    pub nonce: [u8; AesGcm::NONCE_SIZE12],
    pub tag: [u8; AesGcm::TAG_SIZE16],
}

#[repr(C)]
pub struct VEthCipherFrame {
    pub header: VEthCipherFrameHeader,
    pub data: [u8; crate::CIPHER_FRAME_DATA_SIZE as _],
}

#[repr(C)]
pub union VEthFrame {
    pub plain: VEthPlainFrame,
    pub cipher: VEthCipherFrame,
}

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_adapter_create_tx_queue(
    adapter: win::NETADAPTER,
    tx_queue_init: *mut win::NETTXQUEUE_INIT,
) -> win::NTSTATUS {
    trace_entry!("evt_adapter_create_tx_queue");

    let status = (|| {
        let mut tx_config = win::NET_PACKET_QUEUE_CONFIG_INIT(
            Some(send::evt_tx_queue_advance),
            Some(send::evt_tx_queue_set_notification_enabled),
            Some(send::evt_tx_queue_cancel),
        );
        tx_config.evt_start = Some(send::evt_tx_queue_start);
        let mut tx_attributes = win::WDF_OBJECT_ATTRIBUTES_INIT();
        tx_attributes.evt_destroy_callback = Some(send::evt_tx_queue_destroy);
        tx_attributes.context_type_info = send::veth_get_tx_queue_context_type();
        let mut tx_queue = MaybeUninit::uninit();
        let status = unsafe {
            win::NetTxQueueCreate(
                tx_queue_init,
                &tx_attributes,
                &tx_config,
                tx_queue.as_mut_ptr(),
            )
        };
        if !win::NT_SUCCESS(status) {
            trace_exit_status!("NetTxQueueCreate", status);
            return status;
        }
        let tx_queue = unsafe { tx_queue.assume_init() };
        let adapter = VEthAdapter::from_adapter_mut(adapter);
        if let Err(status) = adapter.init_tx_queue(tx_queue) {
            return status;
        }
        win::STATUS_SUCCESS
    })();

    trace_exit_status!("evt_adapter_create_tx_queue", status);
    status
}

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_adapter_create_rx_queue(
    adapter: win::NETADAPTER,
    rx_queue_init: *mut win::NETRXQUEUE_INIT,
) -> win::NTSTATUS {
    trace_entry!("evt_adapter_create_rx_queue");

    let status = (|| {
        let mut rx_config = win::NET_PACKET_QUEUE_CONFIG_INIT(
            Some(recv::evt_rx_queue_advance),
            Some(recv::evt_rx_queue_set_notification_enabled),
            Some(recv::evt_rx_queue_cancel),
        );
        rx_config.evt_start = Some(recv::evt_rx_queue_start);
        let mut rx_attributes = win::WDF_OBJECT_ATTRIBUTES_INIT();
        rx_attributes.evt_destroy_callback = Some(recv::evt_rx_queue_destroy);
        rx_attributes.context_type_info = recv::veth_get_rx_queue_context_type();
        let mut rx_queue = MaybeUninit::uninit();
        let status = unsafe {
            win::NetRxQueueCreate(
                rx_queue_init,
                &rx_attributes,
                &rx_config,
                rx_queue.as_mut_ptr(),
            )
        };
        if !win::NT_SUCCESS(status) {
            trace_exit_status!("NetRxQueueCreate", status);
            return status;
        }
        let rx_queue = unsafe { rx_queue.assume_init() };
        let adapter = VEthAdapter::from_adapter_mut(adapter);
        if let Err(status) = adapter.init_rx_queue(rx_queue) {
            return status;
        }
        win::STATUS_SUCCESS
    })();

    trace_exit_status!("evt_adapter_create_rx_queue", status);
    status
}

#[irql_requires_max(PASSIVE_LEVEL)]
pub extern "system" fn evt_set_packet_filter(
    _adapter: win::NETADAPTER,
    _packet_filter: win::NET_PACKET_FILTER_FLAGS,
) {
    trace_entry!("evt_set_packet_filter");
}
