use core::{
    default::default,
    mem::{self, MaybeUninit},
    ptr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
};

use crate::{
    crypto::AesGcm,
    linked::{LinkedCountedQueue, LinkedIter, LinkedQueue},
    ndis::NblPool,
    os::{
        memory::Lookaside,
        request::SyncRequest,
        socket::UdpSocket,
        sync::{AutoEvent, SpinLock},
        thread::Thread,
    },
    windows::{
        km::{
            ndis::{
                nbl::NET_BUFFER_LIST, NdisMIndicateStatusEx, NdisMSendNetBufferListsComplete,
                NDIS_SIZEOF_STATUS_INDICATION_REVISION_1, NDIS_STATUS_INDICATION,
                NDIS_STATUS_INDICATION_REVISION_1, NDIS_STATUS_LINK_STATE,
            },
            wdm::{
                MmBuildMdlForNonPagedPool, MmInitializeMdl, ADDRESS_AND_SIZE_TO_SPAN_PAGES, MDL,
                PAGE_SIZE, PFN_NUMBER,
            },
        },
        shared::{
            ndis::{
                ndisport::NDIS_DEFAULT_PORT_NUMBER, objectheader::NDIS_OBJECT_HEADER,
                types::NDIS_HANDLE,
            },
            ntddndis::{
                NDIS_LINK_STATE, NDIS_LINK_STATE_REVISION_1, NDIS_MEDIA_CONNECT_STATE,
                NDIS_MEDIA_DUPLEX_STATE, NDIS_OBJECT_TYPE_DEFAULT,
                NDIS_OBJECT_TYPE_STATUS_INDICATION, NDIS_SIZEOF_LINK_STATE_REVISION_1,
                NDIS_SUPPORTED_PAUSE_FUNCTIONS,
            },
            ntstatus::STATUS_SUCCESS,
            ws2ipdef::SOCKADDR_IN6,
        },
        Result,
    },
    NbExt,
};

const MAX_ETH_MTU_SIZE: u32 = 1500;
const MAX_IP_HEADER_SIZE: u32 = 60;
const MAX_UDP_HEADER_SIZE: u32 = 8;
const MAX_ETH_HEADER_SIZE: u32 = 14;

pub const MIN_FRAME_SIZE: u32 = MAX_ETH_HEADER_SIZE;
pub const MAX_FRAME_SIZE: u32 = MAX_ETH_MTU_SIZE - MAX_IP_HEADER_SIZE - MAX_UDP_HEADER_SIZE; // 1432

pub const MAX_FRAME_DATA_SIZE: u32 = MAX_FRAME_SIZE; // 1432
pub const MAX_FRAME_MTU_SIZE: u32 = MAX_FRAME_DATA_SIZE - MAX_ETH_HEADER_SIZE; // 1418

const PLAIN_FRAME_HEADER_SIZE: u32 = 0;
const PLAIN_FRAME_DATA_SIZE: u32 = MAX_FRAME_DATA_SIZE - PLAIN_FRAME_HEADER_SIZE; // 1432
const PLAIN_FRAME_MTU_SIZE: u32 = MAX_FRAME_MTU_SIZE - PLAIN_FRAME_HEADER_SIZE; // 1418

const CIPHER_FRAME_HEADER_SIZE: u32 = mem::size_of::<CipherFrameHeader>() as u32; // 28
const CIPHER_FRAME_DATA_SIZE: u32 = PLAIN_FRAME_DATA_SIZE - CIPHER_FRAME_HEADER_SIZE; // 1404
const CIPHER_FRAME_MTU_SIZE: u32 = PLAIN_FRAME_MTU_SIZE - CIPHER_FRAME_HEADER_SIZE; // 1390

pub struct Adapter {
    pub handle: NDIS_HANDLE,

    local_socket_addr: SOCKADDR_IN6,
    remote_socket_addr: SOCKADDR_IN6,
    pub remote_secret_key: Option<AesGcm>,

    pub socket: Option<UdpSocket>,
    request: SyncRequest,
    tx_request: SyncRequest,

    pub tx_queue: SpinLock<LinkedQueue<NET_BUFFER_LIST>>,
    pub tx_event: AutoEvent,
    tx_canceled: AtomicBool,
    tx_thread: Option<Thread>,

    pub rx_queue: SpinLock<LinkedQueue<NET_BUFFER_LIST>>,
    rx_canceled: AtomicBool,

    tx_ctx_pool: Lookaside<()>,
    rx_nbl_pool: NblPool,

    pub tx_bytes_unicast: AtomicU64,
    pub tx_bytes_multicast: AtomicU64,
    pub tx_bytes_broadcast: AtomicU64,
    pub tx_frames_unicast: AtomicU64,
    pub tx_frames_multicast: AtomicU64,
    pub tx_frames_broadcast: AtomicU64,
    pub tx_errors: AtomicU64,
    pub tx_discards: AtomicU64,

    pub rx_bytes_unicast: AtomicU64,
    pub rx_bytes_multicast: AtomicU64,
    pub rx_bytes_broadcast: AtomicU64,
    pub rx_frames_unicast: AtomicU64,
    pub rx_frames_multicast: AtomicU64,
    pub rx_frames_broadcast: AtomicU64,
    pub rx_errors: AtomicU64,
    pub rx_discards: AtomicU64,
}

impl Adapter {
    const RX_CAPACITY: usize = 1024;
    const TX_POOL_TAG: u32 = u32::from_be_bytes([b'N', b'V', b'N', b'T']);
    const RX_POOL_TAG: u32 = u32::from_be_bytes([b'N', b'V', b'N', b'R']);

    pub unsafe fn init<'a>(uninit: *mut Self, handle: NDIS_HANDLE) -> Result<&'a mut Self> {
        Lookaside::init(ptr::addr_of_mut!((*uninit).tx_ctx_pool), Self::TX_POOL_TAG)?;

        ptr::addr_of_mut!((*uninit).handle).write(handle);

        ptr::addr_of_mut!((*uninit).remote_secret_key).write(None);

        SyncRequest::init(ptr::addr_of_mut!((*uninit).request));
        SyncRequest::init(ptr::addr_of_mut!((*uninit).tx_request));

        SpinLock::init(ptr::addr_of_mut!((*uninit).tx_queue), LinkedQueue::init);
        AutoEvent::init(ptr::addr_of_mut!((*uninit).tx_event));
        ptr::addr_of_mut!((*uninit).tx_canceled).write(AtomicBool::new(false));
        ptr::addr_of_mut!((*uninit).tx_thread).write(None);

        SpinLock::init(ptr::addr_of_mut!((*uninit).rx_queue), LinkedQueue::init);
        ptr::addr_of_mut!((*uninit).rx_canceled).write(AtomicBool::new(false));

        let nbl_pool = NblPool::new(handle, Self::RX_POOL_TAG)?;

        let result = (|| {
            for _ in 0..Self::RX_CAPACITY {
                let nbl = nbl_pool.alloc(ptr::null_mut(), 0, 0)?;
                (*uninit).rx_queue.get_mut().enqueue(nbl);
            }
            Ok(())
        })();
        if let Err(status) = result {
            loop {
                let nbl = (*uninit).rx_queue.get_mut().dequeue();
                if nbl.is_null() {
                    break;
                }
                nbl_pool.dealloc(nbl);
            }
            return Err(status);
        }

        ptr::addr_of_mut!((*uninit).rx_nbl_pool).write(nbl_pool);

        ptr::addr_of_mut!((*uninit).tx_bytes_unicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).tx_bytes_multicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).tx_bytes_broadcast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).tx_frames_unicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).tx_frames_multicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).tx_frames_broadcast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).tx_errors).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).tx_discards).write(AtomicU64::new(0));

        ptr::addr_of_mut!((*uninit).rx_bytes_unicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).rx_bytes_multicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).rx_bytes_broadcast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).rx_frames_unicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).rx_frames_multicast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).rx_frames_broadcast).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).rx_errors).write(AtomicU64::new(0));
        ptr::addr_of_mut!((*uninit).rx_discards).write(AtomicU64::new(0));

        Ok(&mut *uninit)
    }

    pub fn set_local_endpoint(&mut self, endpoint: SOCKADDR_IN6) {
        self.local_socket_addr = endpoint;
    }

    pub fn set_remote_endpoint(&mut self, endpoint: SOCKADDR_IN6) {
        self.remote_socket_addr = endpoint;
    }

    pub fn set_remote_secret_key(&mut self, secret_key: [u8; 16]) -> Result<()> {
        let _secret_key = AesGcm::new(secret_key)?;
        Ok(())
    }

    fn set_link_state(&self, connected: bool) {
        let mut link_state = NDIS_LINK_STATE {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_DEFAULT,
                Revision: NDIS_LINK_STATE_REVISION_1,
                Size: NDIS_SIZEOF_LINK_STATE_REVISION_1,
            },
            MediaConnectState: if connected {
                NDIS_MEDIA_CONNECT_STATE::MediaConnectStateConnected
            } else {
                NDIS_MEDIA_CONNECT_STATE::MediaConnectStateDisconnected
            },
            MediaDuplexState: NDIS_MEDIA_DUPLEX_STATE::MediaDuplexStateFull,
            XmitLinkSpeed: crate::LINK_SPEED,
            RcvLinkSpeed: crate::LINK_SPEED,
            PauseFunctions: NDIS_SUPPORTED_PAUSE_FUNCTIONS::NdisPauseFunctionsUnsupported,
            ..default()
        };
        let mut status_indication = NDIS_STATUS_INDICATION {
            Header: NDIS_OBJECT_HEADER {
                Type: NDIS_OBJECT_TYPE_STATUS_INDICATION,
                Revision: NDIS_STATUS_INDICATION_REVISION_1,
                Size: NDIS_SIZEOF_STATUS_INDICATION_REVISION_1,
            },
            SourceHandle: self.handle,
            PortNumber: NDIS_DEFAULT_PORT_NUMBER,
            StatusCode: NDIS_STATUS_LINK_STATE,
            StatusBuffer: &mut link_state as *mut _ as *mut _,
            StatusBufferSize: mem::size_of_val(&link_state) as u32,
            ..default()
        };
        unsafe { NdisMIndicateStatusEx(self.handle, &mut status_indication) };
    }

    pub fn connect(&mut self) -> Result<()> {
        if let None = self.socket {
            let driver = unsafe { crate::DRIVER.assume_init_ref() };
            let socket = UdpSocket::new(
                driver.socket_client(),
                &self.request,
                self as *const _ as *mut _,
            )?;
            socket.set_ipv6_only(&self.request, false)?;
            socket.bind(&self.request, &self.local_socket_addr)?;
            self.socket = Some(socket);
        }
        if let None = self.tx_thread {
            self.tx_thread = Some(Thread::spawn(Self::start_send, self)?);
        }
        self.set_link_state(true);
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.set_link_state(false);
        if let Some(tx_thread) = self.tx_thread.take() {
            self.tx_canceled.store(true, Relaxed);
            self.tx_event.set();
            tx_thread.join();
        }
        if let Some(socket) = self.socket.take() {
            socket.close(&self.request)?;
        }
        Ok(())
    }

    extern "system" fn start_send(&self) -> ! {
        let socket = unsafe { self.socket.as_ref().unwrap_unchecked() };
        let mut i = 0;
        while !self.tx_canceled.load(Relaxed) {
            i += 1;
            let c = i % 1024 == 0;
            let t = || {
                if c {
                    unsafe { crate::windows::km::wdm::KeQueryPerformanceCounter(ptr::null_mut()) }
                } else {
                    unsafe { MaybeUninit::uninit().assume_init() }
                }
            };
            let t0 = t();
            let nbl_chain = self.tx_queue.lock().dequeue_all();
            if nbl_chain.is_null() {
                self.tx_event.wait();
                continue;
            }
            let t1 = t();
            let mut wbl_queue = MaybeUninit::uninit();
            unsafe { LinkedCountedQueue::init(wbl_queue.as_mut_ptr()) };
            let wbl_queue = unsafe { wbl_queue.assume_init_mut() };
            let nbl_iter = unsafe { LinkedIter::new(nbl_chain) }; // r.a
            for nbl in nbl_iter {
                let nb_iter = unsafe { LinkedIter::new(nbl.first_net_buffer) }; // r.a
                for nb in nb_iter {
                    let wbl = unsafe { &mut *nb.wsk_buf_list_mut() };
                    wbl.Next = ptr::null_mut();
                    let wb = &mut wbl.Buffer;
                    wb.Mdl = nb.current_mdl;
                    wb.Offset = nb.current_mdl_offset;
                    wb.Length = nb.data_length as usize;
                    unsafe { wbl_queue.enqueue(wbl) };
                }
            }
            let wbl_chain = unsafe { wbl_queue.chain() };
            let t2 = t();
            if let Some(wbl_chain) = unsafe { wbl_chain.as_mut() } {
                //+
                // let req = unsafe {
                //     crate::os::memory::Memory::<SyncRequest>::alloc_non_paged(1, 1)
                //         .unwrap_unchecked()
                // };
                // unsafe { SyncRequest::init(req.as_ptr()) };
                // let ctx = unsafe {
                //     crate::os::memory::Memory::<(
                //         NDIS_HANDLE,
                //         *mut crate::windows::km::ndis::nbl::NET_BUFFER_LIST,
                //     )>::alloc_non_paged(1, 1)
                //     .unwrap_unchecked()
                // };
                // unsafe {
                //     *ctx.as_ptr() = (self.handle, nbl_chain);
                // }
                // match socket.send_messages_async(
                //     unsafe { &*req.as_ptr() },
                //     wbl_chain,
                //     &self.remote_socket_addr,
                //     ctx.as_ptr().cast(),
                let wbl_count = wbl_queue.count() as u64;
                match socket.send_messages(&self.tx_request, wbl_chain, &self.remote_socket_addr) {
                    Err(_) => {
                        self.tx_errors.fetch_add(wbl_count as u64, Relaxed);
                    }
                    Ok(sent) => {
                        self.tx_bytes_unicast.fetch_add(sent as u64, Relaxed);
                        self.tx_frames_unicast.fetch_add(wbl_count, Relaxed);
                    }
                }
                // mem::forget(req);
                // mem::forget(ctx);
            }
            let t3 = t();
            unsafe { NdisMSendNetBufferListsComplete(self.handle, nbl_chain, 0) };
            let t4 = t();
            if c {
                trace_println!("%lld %lld %lld %lld %lld", t0, t1, t2, t3, t4);
            }
        }
        Thread::exit(STATUS_SUCCESS);
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        loop {
            let nbl = self.rx_queue.get_mut().dequeue();
            if nbl.is_null() {
                break;
            }
            unsafe { self.rx_nbl_pool.dealloc(nbl) };
        }
    }
}

#[repr(C)]
struct PlainFrame {
    data: [u8; PLAIN_FRAME_DATA_SIZE as usize],
}

#[repr(C)]
pub struct CipherFrameHeader {
    pub nonce: [u8; 12],
    pub tag: [u8; 16],
}

#[repr(C)]
struct CipherFrame {
    header: CipherFrameHeader,
    data: [u8; CIPHER_FRAME_DATA_SIZE as usize],
}

#[repr(C)]
struct MdlRepr {
    mdl: MDL,
    _mdlx:
        [PFN_NUMBER; ADDRESS_AND_SIZE_TO_SPAN_PAGES(PAGE_SIZE - 1, mem::size_of::<PlainFrame>())],
}

impl MdlRepr {
    unsafe fn init_non_paged(uninit: *mut Self, addr: *mut PlainFrame) {
        let mdl = uninit.raw_get();
        MmInitializeMdl(mdl, addr.cast(), mem::size_of::<PlainFrame>());
        MmBuildMdlForNonPagedPool(mdl);
    }

    unsafe fn raw_get(self: *mut Self) -> *mut MDL {
        ptr::addr_of_mut!((*self).mdl)
    }
}
