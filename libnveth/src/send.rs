use alloc::vec::Vec;

use core::{
    mem::{self, MaybeUninit},
    ptr, slice,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use sal::*;

use crate::{
    adapter::VEthFrame,
    net::EthHeader,
    os::thread::Thread,
    peer::Peer,
    socket::{IoRequest, UdpSocket, UdpSocketWorker},
    windows::{
        km::{
            wdm::{MmBuildMdlForNonPagedPool, MmInitializeMdl, MmSizeOfMdl, MDL, PAGE_SIZE},
            wsk::WSK_BUF,
        },
        prelude as win,
    },
    worker::{Worker, WorkerState},
};

pub struct VEthTxQueue {
    tx_queue: win::NETPACKETQUEUE,
    rings: *const win::NET_RING_COLLECTION,
    virtual_address_extension: win::NET_EXTENSION,

    notify: AtomicBool,

    worker: VEthTxWorker<'static>,

    state: Worker,
}

impl VEthTxQueue {
    pub fn init<'a>(
        tx_queue: win::NETPACKETQUEUE,
        socket: &'static UdpSocket,
        request: &'static mut IoRequest,
        peers: &'static Vec<Peer>,
    ) -> Result<&'a mut Self, win::NTSTATUS> {
        unsafe {
            let uninit = Self::from_queue_mut_ptr(tx_queue);

            ptr::raw_mut!((*uninit).tx_queue).write(tx_queue);
            ptr::raw_mut!((*uninit).rings).write(win::NetTxQueueGetRingCollection(tx_queue));

            let query = win::NET_EXTENSION_QUERY_INIT(
                win::NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_NAME.as_ptr(),
                win::NET_FRAGMENT_EXTENSION_VIRTUAL_ADDRESS_VERSION_1,
                win::NET_EXTENSION_TYPE::NetExtensionTypeFragment,
            );
            win::NetTxQueueGetExtension(
                tx_queue,
                &query,
                ptr::raw_mut!((*uninit).virtual_address_extension),
            );

            ptr::raw_mut!((*uninit).notify).write(AtomicBool::new(false));

            let state = Worker::init(ptr::raw_mut!((*uninit).state));

            let init = &mut *uninit;
            VEthTxWorker::init(
                ptr::raw_mut!((*uninit).worker),
                init,
                socket,
                request,
                peers,
                state,
            );

            let init = &mut *uninit;
            init.state
                .init_thread(Thread::spawn_mut(veth_tx_worker, &mut init.worker)?);

            Ok(&mut *uninit)
        }
    }

    fn from_queue_mut<'a>(tx_queue: win::NETPACKETQUEUE) -> &'a mut Self {
        unsafe { &mut *Self::from_queue_mut_ptr(tx_queue) }
    }

    fn from_queue_mut_ptr(tx_queue: win::NETPACKETQUEUE) -> *mut Self {
        veth_get_tx_queue_context(tx_queue.into())
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
static mut WDF_VETH_TX_QUEUE_TYPE_INFO: win::WDF_OBJECT_CONTEXT_TYPE_INFO =
    win::WDF_OBJECT_CONTEXT_TYPE_INFO {
        size: mem::size_of::<win::WDF_OBJECT_CONTEXT_TYPE_INFO>() as _,
        context_name: "VEthTxQueue\0".as_ptr(),
        context_size: mem::size_of::<VEthTxQueue>(),
        unique_type: unsafe { &WDF_VETH_TX_QUEUE_TYPE_INFO },
        evt_driver_get_unique_context_type: None,
    };

fn veth_get_tx_queue_context(handle: win::WDFOBJECT) -> *mut VEthTxQueue {
    unsafe {
        win::WdfObjectGetTypedContextWorker(handle, WDF_VETH_TX_QUEUE_TYPE_INFO.unique_type).cast()
    }
}

pub fn veth_get_tx_queue_context_type() -> *const win::WDF_OBJECT_CONTEXT_TYPE_INFO {
    unsafe { &WDF_VETH_TX_QUEUE_TYPE_INFO }
}

#[repr(C)]
union MdlRepr {
    mdl: MDL,
    mdlx: [u8; unsafe { MmSizeOfMdl((PAGE_SIZE - 1) as _, mem::size_of::<VEthFrame>()) }],
}

struct VEthTxWorker<'a> {
    socket: UdpSocketWorker<'a>,

    tx_queue: win::NETPACKETQUEUE,
    rings: *const win::NET_RING_COLLECTION,
    virtual_address_extension: *const win::NET_EXTENSION,

    notify: &'a AtomicBool,

    peers: &'a Vec<Peer>,

    state: &'a mut WorkerState,

    mdl: MaybeUninit<MdlRepr>,
    frame: VEthFrame,
}

impl<'a> VEthTxWorker<'a> {
    unsafe fn init(
        uninit: *mut Self,
        tx: &'a VEthTxQueue,
        socket: &'a UdpSocket,
        request: &'a mut IoRequest,
        peers: &'a Vec<Peer>,
        state: &'a mut WorkerState,
    ) {
        ptr::raw_mut!((*uninit).socket).write(UdpSocketWorker::new(socket, request));

        ptr::raw_mut!((*uninit).tx_queue).write(tx.tx_queue);
        ptr::raw_mut!((*uninit).rings).write(tx.rings);
        ptr::raw_mut!((*uninit).virtual_address_extension).write(&tx.virtual_address_extension);

        ptr::raw_mut!((*uninit).notify).write(&tx.notify);

        ptr::raw_mut!((*uninit).peers).write(peers);

        ptr::raw_mut!((*uninit).state).write(state);
    }
}

extern "system" fn veth_tx_worker(tx: &mut VEthTxWorker) {
    trace_entry!("veth_tx_worker");
    while tx.state.wait_for_start() {
        let packets = unsafe { &mut *win::NetRingCollectionGetPacketRing(tx.rings) };
        let fragments = unsafe { &mut *win::NetRingCollectionGetFragmentRing(tx.rings) };
        while !tx.state.is_canceled() {
            let packet_index = packets.next_index;
            let packet_end_index = packets.end_index;
            if packet_index == packet_end_index {
                if tx.notify.load(Relaxed) {
                    unsafe { win::NetTxQueueNotifyMoreCompletedPacketsAvailable(tx.tx_queue) };
                }
                tx.state.wait_for_work();
                continue;
            }

            let packet = unsafe { &*win::NetRingGetPacketAtIndex(packets, packet_index) };
            let mut fragment_index = packet.fragment_index;
            let fragment_end_index = unsafe {
                win::NetRingAdvanceIndex(fragments, fragment_index, packet.fragment_count.into())
            };
            let mut frame_offset = 0;
            while fragment_index != fragment_end_index {
                let fragment =
                    unsafe { &mut *win::NetRingGetFragmentAtIndex(fragments, fragment_index) };
                let virtual_address = unsafe {
                    &mut *win::NetExtensionGetFragmentVirtualAddress(
                        tx.virtual_address_extension,
                        fragment_index,
                    )
                };
                let virtual_address = virtual_address.virtual_address;
                let length = fragment.valid_length() as _;
                unsafe {
                    tx.frame.data[frame_offset..frame_offset + length].copy_from_slice(
                        slice::from_raw_parts(
                            virtual_address.offset(fragment.offset() as _),
                            length,
                        ),
                    )
                };
                frame_offset += length;
                fragment_index = unsafe { win::NetRingIncrementIndex(fragments, fragment_index) };
            }
            unsafe {
                MmInitializeMdl(
                    ptr::raw_mut!((*tx.mdl.as_mut_ptr()).mdl),
                    tx.frame.data.as_mut_ptr().cast(),
                    frame_offset,
                )
            };
            unsafe { MmBuildMdlForNonPagedPool(ptr::raw_mut!((*tx.mdl.as_mut_ptr()).mdl)) };
            let buf = WSK_BUF {
                mdl: unsafe { &mut tx.mdl.assume_init_mut().mdl },
                offset: 0,
                length: frame_offset,
            };
            let socket = &mut tx.socket;
            let mut send_to = |addr| {
                match socket.send_to(&buf, addr) {
                    Err(_status) => {
                        // TODO
                    }
                    Ok(sent) => {
                        trace_println!("<-- %u", sent);
                    }
                }
            };
            if frame_offset >= mem::size_of::<EthHeader>() {
                let eth = unsafe { &*tx.frame.data.as_ptr().cast::<EthHeader>() };
                let dst = eth.dst();
                if dst.is_multicast() {
                    if dst.is_broadcast() {
                        tx.peers.iter().for_each(|peer| send_to(&peer.socket_addr));
                    }
                } else if let Some(peer) = tx.peers.iter().find(|peer| {
                    if let Some(addr) = peer.mac_addr.read().as_ref() {
                        dst == addr
                    } else {
                        false
                    }
                }) {
                    send_to(&peer.socket_addr);
                }
            }
            packets.next_index = unsafe { win::NetRingIncrementIndex(packets, packet_index) };
        }

        tx.state.signal_stopped();
    }

    trace_exit!("veth_tx_worker");
    tx.state.exit();
}

#[irql_requires(PASSIVE_LEVEL)]
pub extern "system" fn evt_tx_queue_start(tx_queue: win::NETPACKETQUEUE) {
    trace_entry!("evt_tx_queue_start");

    let tx = VEthTxQueue::from_queue_mut(tx_queue);
    tx.start();
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub extern "system" fn evt_tx_queue_advance(tx_queue: win::NETPACKETQUEUE) {
    // trace_entry!("evt_tx_queue_advance");

    let tx = VEthTxQueue::from_queue_mut(tx_queue);

    let rings = tx.rings;
    let packets = unsafe { &mut *win::NetRingCollectionGetPacketRing(rings) };

    trace_entry_args!(
        "evt_tx_queue_advance",
        "%u:%u:%u",
        packets.begin_index,
        packets.next_index,
        packets.end_index,
    );

    let mut packet_index = packets.begin_index;
    let packet_end_index = packets.next_index;
    while packet_index != packet_end_index {
        packet_index = unsafe { win::NetRingIncrementIndex(packets, packet_index) };
    }
    packets.begin_index = packet_index;

    tx.state.signal_work();
}

#[irql_requires(PASSIVE_LEVEL)]
pub extern "system" fn evt_tx_queue_set_notification_enabled(
    tx_queue: win::NETPACKETQUEUE,
    notification_enabled: bool,
) {
    trace_entry!("evt_tx_queue_set_notification_enabled");

    let tx = VEthTxQueue::from_queue_mut(tx_queue);
    tx.notify.store(notification_enabled, Relaxed);
}

#[irql_requires(PASSIVE_LEVEL)]
pub extern "system" fn evt_tx_queue_cancel(tx_queue: win::NETPACKETQUEUE) {
    trace_entry!("evt_tx_queue_cancel");

    let tx = VEthTxQueue::from_queue_mut(tx_queue);
    tx.state.cancel();
    tx.state.wait_for_stopped();

    let rings = tx.rings;
    let packets = unsafe { &mut *win::NetRingCollectionGetPacketRing(rings) };
    packets.begin_index = packets.end_index;

    trace_exit!("evt_tx_queue_cancel");
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub extern "system" fn evt_tx_queue_destroy(object: win::WDFOBJECT) {
    trace_entry!("evt_tx_queue_destroy");

    let tx = unsafe { &mut *veth_get_tx_queue_context(object) };
    tx.drop();

    trace_exit!("evt_tx_queue_destroy");
}
