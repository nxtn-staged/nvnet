use core::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
};

use crate::{
    init::InitGuard,
    windows::{
        km::{
            ndis::*,
            wdm::{
                ExAllocateFromLookasideListEx, ExDeleteLookasideListEx, ExFreeToLookasideListEx,
                ExInitializeLookasideListEx, InitializeSListHead, InterlockedPopEntrySList,
                InterlockedPushEntrySList, LOOKASIDE_LIST_EX, POOL_TYPE, SLIST_ENTRY, SLIST_HEADER,
            },
        },
        shared::ntdef::{NTSTATUS, NT_SUCCESS},
    },
};

struct SingleList(SLIST_HEADER);

impl SingleList {
    pub fn new() -> Self {
        let mut list_head = MaybeUninit::uninit();
        unsafe { InitializeSListHead(list_head.as_mut_ptr()) };
        let list_head = unsafe { list_head.assume_init() };
        Self(list_head)
    }

    pub fn push(&mut self, list_tail: *mut SLIST_ENTRY) {
        unsafe { InterlockedPushEntrySList(&mut self.0, list_tail) };
    }

    pub fn pop(&mut self) -> *mut SLIST_ENTRY {
        unsafe { InterlockedPopEntrySList(&mut self.0) }
    }
}

macro_rules! container_of {
    ($ptr:expr, $member:expr) => {
        $ptr.cast::<u8>()
            .offset(-($member(core::ptr::null_mut()) as isize))
            .cast()
    };
}

pub struct NblList(SingleList);

impl NblList {
    pub fn new() -> Self {
        Self(SingleList::new())
    }

    fn from_nbl(nbl: *mut NET_BUFFER_LIST) -> *mut SLIST_ENTRY {
        unsafe { &mut (*nbl).miniport_reserved as *mut _ as *mut _ }
    }

    fn into_nbl(list_entry: *const SLIST_ENTRY) -> *const NET_BUFFER_LIST {
        unsafe { container_of!(list_entry, Self::from_nbl) }
    }

    pub fn push(&mut self, nbl: *mut NET_BUFFER_LIST) {
        self.0.push(Self::from_nbl(nbl))
    }

    pub fn pop<'a>(&mut self) -> Option<&'a NET_BUFFER_LIST> {
        let list_entry = self.0.pop();
        if list_entry.is_null() {
            None
        } else {
            unsafe { Some(Self::into_nbl(list_entry).as_ref().unwrap()) }
        }
    }
}

pub struct BufPool<T> {
    lookaside: LOOKASIDE_LIST_EX,
    t: PhantomData<T>,
}

impl<T> BufPool<T> {
    pub fn init(uninit: *mut Self, tag: u32) -> Result<InitGuard<Self>, NTSTATUS> {
        unsafe {
            let uninit = uninit.as_mut().unwrap();
            let status = ExInitializeLookasideListEx(
                &mut uninit.lookaside,
                None,
                None,
                POOL_TYPE::NonPagedPool,
                0,
                mem::size_of::<T>(),
                tag,
                0,
            );
            if !NT_SUCCESS(status) {
                Err(status)
            } else {
                Ok(InitGuard::new(uninit))
            }
        }
    }

    pub fn allocate(&mut self) -> *mut T {
        ExAllocateFromLookasideListEx(&mut self.lookaside).cast()
    }

    pub fn free(&mut self, entry: *mut T) {
        ExFreeToLookasideListEx(&mut self.lookaside, entry.cast())
    }
}

impl<T> Drop for BufPool<T> {
    fn drop(&mut self) {
        unsafe { ExDeleteLookasideListEx(&mut self.lookaside) }
    }
}
