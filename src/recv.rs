use core::{
    mem::{self, MaybeUninit},
    ptr,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use sal::*;

use crate::{
    net::{ArpPacket, EthFrameHeader, MacAddr},
    os::thread::Thread,
    socket::{IoRequest, UdpSocket, UdpSocketWorker},
    windows::{
        km::{
            wdm::{MmBuildMdlForNonPagedPool, MmInitializeMdl, MDL},
            wsk::WSK_BUF,
        },
        prelude as win,
        shared::{ntdef::NTSTATUS, ws2ipdef::SOCKADDR_IN6},
    },
    worker::{Worker, WorkerState},
};

pub struct VEthRxQueue {
    rx_queue: win::NETPACKETQUEUE,
    rings: *const win::NET_RING_COLLECTION,
    virtual_address_extension: win::NET_EXTENSION,

    notify: AtomicBool,

    worker: VEthRxWorker<'static>,

    state: Worker,
}

impl VEthRxQueue {
    pub fn init<'a>(
        rx_queue: win::NETPACKETQUEUE,
        socket: &'static UdpSocket,
        request: &'static mut IoRequest,
        remote_addr: &'static SOCKADDR_IN6,
        remote_mac_addr: &'static mut Option<MacAddr>,
    ) -> Result<&'a mut Self, NTSTATUS> {
        unsafe {
            let uninit = Self::from_queue_mut_ptr(rx_queue);

            ptr::raw_mut!((*uninit).rx_queue).write(rx_queue);
            ptr::raw_mut!((*uninit).rings).write(win::NetRxQueueGetRingCollection(rx_queue));

            let query = win::NET_EXTENSION_QUERY_INIT(
                win::NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_NAME.as_ptr(),
                win::NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_VERSION_1,
                win::NET_EXTENSION_TYPE::NetExtensionTypeFragment,
            );
            win::NetRxQueueGetExtension(
                rx_queue,
                &query,
                ptr::raw_mut!((*uninit).virtual_address_extension),
            );

            ptr::raw_mut!((*uninit).notify).write(AtomicBool::new(false));

            let state = Worker::init(ptr::raw_mut!((*uninit).state));

            let init = &mut *uninit;
            VEthRxWorker::init(
                ptr::raw_mut!((*uninit).worker),
                init,
                socket,
                request,
                remote_addr,
                remote_mac_addr,
                state,
            );

            let init = &mut *uninit;
            init.state
                .init_thread(Thread::spawn_mut(veth_rx_worker, &mut init.worker)?);

            Ok(&mut *uninit)
        }
    }

    fn from_queue_mut<'a>(rx_queue: win::NETPACKETQUEUE) -> &'a mut Self {
        unsafe { &mut *Self::from_queue_mut_ptr(rx_queue) }
    }

    fn from_queue_mut_ptr(rx_queue: win::NETPACKETQUEUE) -> *mut Self {
        veth_get_rx_queue_context(rx_queue.into())
    }

    pub fn start(&mut self) {
        self.state.start();
    }

    fn drop(&mut self) {
        self.state.terminate();

        unsafe { ptr::drop_in_place(self) }
    }
}

// WDF_DECLARE_CONTEXT_TYPE_WITH_NAME
static mut WDF_VETH_RX_QUEUE_TYPE_INFO: win::WDF_OBJECT_CONTEXT_TYPE_INFO =
    win::WDF_OBJECT_CONTEXT_TYPE_INFO {
        size: mem::size_of::<win::WDF_OBJECT_CONTEXT_TYPE_INFO>() as _,
        context_name: "VEthRxQueue\0".as_ptr(),
        context_size: mem::size_of::<VEthRxQueue>(),
        unique_type: unsafe { &WDF_VETH_RX_QUEUE_TYPE_INFO },
        evt_driver_get_unique_context_type: None,
    };

fn veth_get_rx_queue_context(handle: win::WDFOBJECT) -> *mut VEthRxQueue {
    unsafe {
        win::WdfObjectGetTypedContextWorker(handle, WDF_VETH_RX_QUEUE_TYPE_INFO.unique_type).cast()
    }
}

pub fn veth_get_rx_queue_context_type() -> *const win::WDF_OBJECT_CONTEXT_TYPE_INFO {
    unsafe { &WDF_VETH_RX_QUEUE_TYPE_INFO }
}

struct VEthRxWorker<'a> {
    socket: UdpSocketWorker<'a>,

    rx_queue: win::NETPACKETQUEUE,
    rings: *const win::NET_RING_COLLECTION,
    virtual_address_extension: *const win::NET_EXTENSION,

    notify: &'a AtomicBool,

    remote_addr: &'a SOCKADDR_IN6,
    remote_mac_addr: &'a mut Option<MacAddr>,

    state: &'a mut WorkerState,

    mdl: MaybeUninit<MDL>,
    addr: MaybeUninit<SOCKADDR_IN6>,
}

impl<'a> VEthRxWorker<'a> {
    unsafe fn init(
        uninit: *mut Self,
        rx: &'a VEthRxQueue,
        socket: &'a UdpSocket,
        request: &'a mut IoRequest,
        remote_addr: &'a SOCKADDR_IN6,
        remote_mac_addr: &'a mut Option<MacAddr>,
        state: &'a mut WorkerState,
    ) {
        ptr::raw_mut!((*uninit).socket).write(UdpSocketWorker::new(socket, request));

        ptr::raw_mut!((*uninit).rx_queue).write(rx.rx_queue);
        ptr::raw_mut!((*uninit).rings).write(rx.rings);
        ptr::raw_mut!((*uninit).virtual_address_extension).write(&rx.virtual_address_extension);

        ptr::raw_mut!((*uninit).notify).write(&rx.notify);

        ptr::raw_mut!((*uninit).remote_addr).write(remote_addr);
        ptr::raw_mut!((*uninit).remote_mac_addr).write(remote_mac_addr);

        ptr::raw_mut!((*uninit).state).write(state);
    }
}

extern "system" fn veth_rx_worker(rx: &mut VEthRxWorker) {
    trace_entry!("veth_rx_worker");
    while rx.state.wait_for_start() {
        let fragments = unsafe { &mut *win::NetRingCollectionGetFragmentRing(rx.rings) };
        while !rx.state.is_canceled() {
            let fragment_index = fragments.next_index;
            let fragment_end_index = fragments.end_index;
            if fragment_index == fragment_end_index {
                if rx.notify.load(Relaxed) {
                    unsafe { win::NetRxQueueNotifyMoreReceivedPacketsAvailable(rx.rx_queue) };
                }
                rx.state.wait_for_work();
                continue;
            }

            let fragment =
                unsafe { &mut *win::NetRingGetFragmentAtIndex(fragments, fragment_index) };
            let length = fragment.capacity() as _;
            let virtual_address = unsafe {
                &mut *win::NetExtensionGetFragmentVirtualAddress(
                    rx.virtual_address_extension,
                    fragment_index,
                )
            };
            let virtual_address = virtual_address.virtual_address;
            unsafe { MmInitializeMdl(rx.mdl.as_mut_ptr(), virtual_address, length) };
            unsafe { MmBuildMdlForNonPagedPool(rx.mdl.as_mut_ptr()) };
            let buf = WSK_BUF {
                mdl: unsafe { rx.mdl.assume_init_mut() },
                offset: 0,
                length,
            };
            match rx.socket.recv_from(&buf, &mut rx.addr) {
                Err(_status) => {
                    // TODO
                }
                Ok(received) => {
                    trace_println!("--> %u", received);
                    fragment.set_valid_length(received as _);
                    fragment.set_offset(0);

                    if received >= mem::size_of::<EthFrameHeader>() + mem::size_of::<ArpPacket>() {
                        let eth = unsafe { &*virtual_address.cast::<EthFrameHeader>() };
                        if eth.is_arp() {
                            let arp = unsafe {
                                &*virtual_address
                                    .cast::<u8>()
                                    .offset(mem::size_of::<EthFrameHeader>() as _)
                                    .cast::<ArpPacket>()
                            };
                            if arp.is_reply() {
                                rx.remote_mac_addr.replace(arp.src_mac());
                            }
                        }
                    }
                }
            }
            fragments.next_index = unsafe { win::NetRingIncrementIndex(fragments, fragment_index) };
            // TODO
            if rx.notify.load(Relaxed) {
                unsafe { win::NetRxQueueNotifyMoreReceivedPacketsAvailable(rx.rx_queue) };
            }
        }

        rx.state.signal_stopped();
    }

    trace_exit!("veth_rx_worker");
    rx.state.exit();
}

#[irql_requires(PASSIVE_LEVEL)]
pub extern "system" fn evt_rx_queue_start(rx_queue: win::NETPACKETQUEUE) {
    trace_entry!("evt_rx_queue_start");

    let rx = VEthRxQueue::from_queue_mut(rx_queue);
    rx.start();
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub extern "system" fn evt_rx_queue_advance(rx_queue: win::NETPACKETQUEUE) {
    // trace_entry!("evt_rx_queue_advance");

    let rx = VEthRxQueue::from_queue_mut(rx_queue);

    let rings = rx.rings;
    let packets = unsafe { &mut *win::NetRingCollectionGetPacketRing(rings) };
    let fragments = unsafe { &mut *win::NetRingCollectionGetFragmentRing(rings) };

    trace_entry_args!(
        "evt_rx_queue_advance",
        "%u:%u:%u",
        fragments.begin_index,
        fragments.next_index,
        fragments.end_index,
    );

    let mut packet_index = packets.begin_index;
    let mut fragment_index = fragments.begin_index;
    let fragment_end_index = fragments.next_index;
    while fragment_index != fragment_end_index {
        let packet = unsafe { &mut *win::NetRingGetPacketAtIndex(packets, packet_index) };
        packet.fragment_index = fragment_index;
        packet.fragment_count = 1;
        packet_index = unsafe { win::NetRingIncrementIndex(packets, packet_index) };
        fragment_index = unsafe { win::NetRingIncrementIndex(fragments, fragment_index) };
    }
    packets.begin_index = packet_index;
    fragments.begin_index = fragment_index;

    rx.state.signal_work();
}

#[irql_requires(PASSIVE_LEVEL)]
pub extern "system" fn evt_rx_queue_set_notification_enabled(
    rx_queue: win::NETPACKETQUEUE,
    notification_enabled: bool,
) {
    trace_entry!("evt_rx_queue_set_notification_enabled");

    let rx = VEthRxQueue::from_queue_mut(rx_queue);
    rx.notify.store(notification_enabled, Relaxed);
}

#[irql_requires(PASSIVE_LEVEL)]
pub extern "system" fn evt_rx_queue_cancel(rx_queue: win::NETPACKETQUEUE) {
    trace_entry!("evt_rx_queue_cancel");

    let rx = VEthRxQueue::from_queue_mut(rx_queue);
    rx.state.cancel();
    rx.worker.socket.request.cancel();
    rx.state.wait_for_stopped();

    let rings = rx.rings;
    let packets = unsafe { &mut *win::NetRingCollectionGetPacketRing(rings) };
    packets.begin_index = packets.end_index;
    let fragments = unsafe { &mut *win::NetRingCollectionGetFragmentRing(rings) };
    fragments.begin_index = fragments.end_index;

    trace_exit!("evt_rx_queue_cancel");
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub extern "system" fn evt_rx_queue_destroy(object: win::WDFOBJECT) {
    trace_entry!("evt_rx_queue_destroy");

    let rx = unsafe { &mut *veth_get_rx_queue_context(object) };
    rx.drop();

    trace_exit!("evt_rx_queue_destroy");
}
