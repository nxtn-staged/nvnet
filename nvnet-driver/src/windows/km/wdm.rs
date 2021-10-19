use core::{ffi::c_void, mem, ptr};

use crate::windows::shared::{
    dpfilter::DPFLTR_TYPE,
    ntdef::{EVENT_TYPE, HANDLE, KIRQL, LARGE_INTEGER, LIST_ENTRY, NTSTATUS, OBJECT_ATTRIBUTES},
};

// L84
c_type!(
    pub struct EPROCESS;
);

c_type!(
    pub struct ETHREAD;
);

// L286
pub type KPRIORITY = i32;

c_type!(
    pub struct KSPIN_LOCK(usize);
);

// L6048
c_type!(
    pub struct SECURITY_DESCRIPTOR;
);

// L7449
c_type!(
    pub struct IO_STATUS_BLOCK {
        pub Status: NTSTATUS, // ...
        pub Information: usize,
    }
);

// L7515
c_type!(
    pub type PIO_APC_ROUTINE = fn() -> !;
);

// L7965
c_type!(
    pub enum INTERFACE_TYPE {
        Internal = 0,
    }
);

// L8712
pub const THREAD_ALL_ACCESS: u32 = 0x000F0000 | 0x00100000 | 0xFFFF;

c_type!(
    pub struct CLIENT_ID;
);

// L12306
c_type!(
    #[repr(align(16))]
    pub struct SLIST_ENTRY {
        pub Next: *mut SLIST_ENTRY,
    }
);

c_type!(
    #[repr(align(16))]
    pub struct SLIST_HEADER {
        _unused: [u64; 2],
    }
);

// L12469
pub const FAST_FAIL_FATAL_APP_EXIT: u32 = 7;

pub unsafe fn __fastfail(Code: u32) -> ! {
    extern "system" {
        fn vnet_fastfail(Code: u32) -> !;
    }
    vnet_fastfail(Code);
}

// L14150
extern "C" {
    pub fn DbgPrintEx(ComponentId: DPFLTR_TYPE, Level: u32, Format: *const u8, ...) -> u32;
}

// L17351
c_type!(
    pub type PALLOCATE_FUNCTION_EX = fn(
        PoolType: POOL_TYPE,
        NumberOfBytes: usize,
        Tag: u32,
        Lookaside: *mut LOOKASIDE_LIST_EX,
    ) -> *mut c_void;
);

c_type!(
    pub type PFREE_FUNCTION_EX = fn(Buffer: *mut c_void, Lookaside: *mut LOOKASIDE_LIST_EX);
);

// L17453
c_type!(
    pub struct GENERAL_LOOKASIDE_POOL {
        pub ListHead: SLIST_HEADER, // ...
        pub Depth: u16,
        pub MaximumDepth: u16,
        pub TotalAllocates: u32,
        pub AllocateMisses: u32, // ...
        pub TotalFrees: u32,
        pub FreeMisses: u32, // ...
        pub Type: POOL_TYPE,
        pub Tag: u32,
        pub Size: u32,
        pub AllocateEx: PALLOCATE_FUNCTION_EX, // ...
        pub FreeEx: PFREE_FUNCTION_EX,         // ...
        pub ListEntry: LIST_ENTRY,
        pub LastTotalAllocates: u32,
        pub LastAllocateMisses: u32, // ...
        pub Future: [u32; 2],
    }
);

// L17479
c_type!(
    #[repr(i8)]
    pub enum KPROCESSOR_MODE {
        KernelMode = 0,
    }
);

// L17667
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SHIFT: usize = 12;

// L17747
c_type!(
    pub struct MDL {
        pub Next: *mut MDL,
        pub Size: i16,
        pub MdlFlags: i16,
        pub Process: *mut EPROCESS,
        pub MappedSystemVa: *mut c_void,
        pub StartVa: *mut c_void,
        pub ByteCount: u32,
        pub ByteOffset: u32,
    }
);

// L18026
c_type!(
    pub struct KEVENT {
        _unused: [u8; 24],
    }
);

// L18845
c_type!(
    pub struct PFN_NUMBER(pub u64);
);

// L21861
c_type!(
    pub enum KWAIT_REASON {
        Executive = 0,
    }
);

// L21942
c_type!(
    pub type PKSTART_ROUTINE = fn(StartContext: *mut c_void);
);

// L22262
extern "system" {
    pub fn KeInitializeEvent(Event: *mut KEVENT, Type: EVENT_TYPE, State: bool);

    // #[irql_requires_max(DISPATCH_LEVEL)]
    // ...
    pub fn KeSetEvent(Event: *mut KEVENT, Increment: KPRIORITY, Wait: bool) -> i32;
}

// L22671
extern "system" {
    // #[irql_requires_max(APC_LEVEL)]
    // ...
    pub fn KeWaitForSingleObject(
        Object: *mut c_void,
        WaitReason: KWAIT_REASON,
        WaitMode: KPROCESSOR_MODE,
        Alertable: bool,
        Timeout: *mut LARGE_INTEGER,
    ) -> NTSTATUS;

    pub fn KeInitializeSpinLock(SpinLock: *mut KSPIN_LOCK);
}

pub unsafe fn KeAcquireSpinLock(SpinLock: *mut KSPIN_LOCK, OldIrql: *mut KIRQL) {
    *OldIrql = KeAcquireSpinLockRaiseToDpc(SpinLock);
}

extern "system" {
    // #[irql_requires_min(DISPATCH_LEVEL)]
    pub fn KeAcquireSpinLockAtDpcLevel(SpinLock: *mut KSPIN_LOCK);

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn KeAcquireSpinLockRaiseToDpc(SpinLock: *mut KSPIN_LOCK) -> KIRQL;

    // #[irql_requires(DISPATCH_LEVEL)]
    pub fn KeReleaseSpinLock(SpinLock: *mut KSPIN_LOCK, NewIrql: KIRQL);

    // #[irql_requires_min(DISPATCH_LEVEL)]
    pub fn KeReleaseSpinLockFromDpcLevel(SpinLock: *mut KSPIN_LOCK);
}

// L24055
c_type!(
    pub enum POOL_TYPE {
        NonPagedPool = 0,
    }
);

extern "system" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    // ...
    pub fn ExAllocatePoolWithTag(
        PoolType: POOL_TYPE,
        NumberOfBytes: usize,
        Tag: u32,
    ) -> *mut c_void;
}

// #[irql_requires_max(DISPATCH_LEVEL)]
// ...
pub unsafe fn ExAllocatePoolUninitialized(
    PoolType: POOL_TYPE,
    NumberOfBytes: usize,
    Tag: u32,
) -> *mut c_void {
    ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag)
}

// L24761
extern "system" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn ExFreePoolWithTag(P: *mut c_void, Tag: u32);
}

// L25134
extern "system" {
    pub fn ExQueryDepthSList(SListHead: *mut SLIST_HEADER) -> u16;
}

pub unsafe fn InterlockedPopEntrySList(Head: *mut SLIST_HEADER) -> *mut SLIST_ENTRY {
    ExpInterlockedPopEntrySList(Head)
}

pub unsafe fn InterlockedPushEntrySList(
    Head: *mut SLIST_HEADER,
    Entry: *mut SLIST_ENTRY,
) -> *mut SLIST_ENTRY {
    ExpInterlockedPushEntrySList(Head, Entry)
}

extern "system" {
    pub fn ExpInterlockedPopEntrySList(ListHead: *mut SLIST_HEADER) -> *mut SLIST_ENTRY;

    pub fn ExpInterlockedPushEntrySList(
        ListHead: *mut SLIST_HEADER,
        ListEntry: *mut SLIST_ENTRY,
    ) -> *mut SLIST_ENTRY;
}

// L25308
c_type!(
    pub struct LOOKASIDE_LIST_EX {
        pub L: GENERAL_LOOKASIDE_POOL,
    }
);

extern "system" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn ExInitializeLookasideListEx(
        Lookaside: *mut LOOKASIDE_LIST_EX,
        Allocate: PALLOCATE_FUNCTION_EX,
        Free: PFREE_FUNCTION_EX,
        PoolType: POOL_TYPE,
        Flags: u32,
        Size: usize,
        Tag: u32,
        Depth: u16,
    ) -> NTSTATUS;

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn ExDeleteLookasideListEx(Lookaside: *mut LOOKASIDE_LIST_EX);
}

// #[irql_requires_max(DISPATCH_LEVEL)]
pub unsafe fn ExAllocateFromLookasideListEx(Lookaside: *mut LOOKASIDE_LIST_EX) -> *mut c_void {
    (*Lookaside).L.TotalAllocates += 1;
    let mut entry = InterlockedPopEntrySList(&mut (*Lookaside).L.ListHead).cast::<c_void>();
    if entry.is_null() {
        (*Lookaside).L.AllocateMisses += 1;
        entry = (*Lookaside).L.AllocateEx.unwrap_unchecked()(
            (*Lookaside).L.Type,
            (*Lookaside).L.Size as usize,
            (*Lookaside).L.Tag,
            Lookaside,
        );
    }
    entry
}

// #[irql_requires_max(DISPATCH_LEVEL)]
pub unsafe fn ExFreeToLookasideListEx(Lookaside: *mut LOOKASIDE_LIST_EX, Entry: *mut c_void) {
    (*Lookaside).L.TotalFrees += 1;
    if ExQueryDepthSList(&mut (*Lookaside).L.ListHead) >= (*Lookaside).L.Depth {
        (*Lookaside).L.FreeMisses += 1;
        (*Lookaside).L.FreeEx.unwrap_unchecked()(Entry, Lookaside);
    } else {
        InterlockedPushEntrySList(&mut (*Lookaside).L.ListHead, Entry.cast());
    }
}

// L27786
pub const IO_NO_INCREMENT: i8 = 0;

// 28008
pub const fn BYTE_OFFSET(Va: usize) -> usize {
    Va & (PAGE_SIZE - 1)
}

pub fn PAGE_ALIGN(Va: *mut c_void) -> *mut c_void {
    ((Va as usize) & !(PAGE_SIZE - 1)) as *mut _
}

// L28059
pub const fn ADDRESS_AND_SIZE_TO_SPAN_PAGES(Va: usize, Size: usize) -> usize {
    (BYTE_OFFSET(Va) + Size + (PAGE_SIZE - 1)) >> PAGE_SHIFT
}

// L28296
extern "system" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn MmBuildMdlForNonPagedPool(MemoryDescriptorList: *mut MDL);
}

// L28979
// #[irql_requires_max(DISPATCH_LEVEL)]
pub unsafe fn MmInitializeMdl(MemoryDescriptorList: *mut MDL, BaseVa: *mut c_void, Length: usize) {
    (*MemoryDescriptorList).Next = ptr::null_mut();
    (*MemoryDescriptorList).Size = (mem::size_of::<MDL>()
        + mem::size_of::<PFN_NUMBER>() * ADDRESS_AND_SIZE_TO_SPAN_PAGES(BaseVa as usize, Length))
        as i16;
    (*MemoryDescriptorList).MdlFlags = 0;
    (*MemoryDescriptorList).StartVa = PAGE_ALIGN(BaseVa);
    (*MemoryDescriptorList).ByteOffset = BYTE_OFFSET(BaseVa as usize) as u32;
    (*MemoryDescriptorList).ByteCount = Length as u32;
}

// L29528
extern "system" {
    // #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn PsCreateSystemThread(
        ThreadHandle: *mut HANDLE,
        DesiredAccess: u32,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES,
        ProcessHandle: HANDLE,
        ClientId: *mut CLIENT_ID,
        StartRoutine: PKSTART_ROUTINE,
        StartContext: *mut c_void,
    ) -> NTSTATUS;

    // #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn PsTerminateSystemThread(ExitStatus: NTSTATUS) -> NTSTATUS;
}

// L29646
pub const IRP_MJ_CREATE: usize = 0x00;
pub const IRP_MJ_CLOSE: usize = 0x02;
pub const IRP_MJ_DEVICE_CONTROL: usize = 0x0e;
pub const IRP_MJ_MAXIMUM_FUNCTION: usize = 0x1b;

// L29923
c_type!(
    pub type PDRIVER_CANCEL = fn() -> !;
);

c_type!(
    // #[irql_requires_max(DISPATCH_LEVEL)]
    // #[irql_requires_same]
    pub type PDRIVER_DISPATCH = fn(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP) -> NTSTATUS;
);

c_type!(
    // #[irql_requires_max(PASSIVE_LEVEL)]
    // #[irql_requires_same]
    pub type PDRIVER_DISPATCH_PAGED =
        fn(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP) -> NTSTATUS;
);

// L30671
c_type!(
    pub struct DEVICE_OBJECT;
);

// L30795
c_type!(
    pub struct DRIVER_OBJECT;
);

// L30933
c_type!(
    pub struct FILE_OBJECT;
);

// L31006
c_type!(
    #[repr(align(16))]
    pub struct IRP {
        pub Type: i16,
        pub Size: u16,
        pub MdlAddress: *mut MDL,
        pub Flags: u32,
        pub SystemBuffer: *mut c_void, // ...
        pub ThreadListEntry: LIST_ENTRY,
        pub IoStatus: IO_STATUS_BLOCK,
        pub RequestorMode: KPROCESSOR_MODE,
        pub PendingReturned: bool,
        pub StackCount: i8,
        pub CurrentLocation: i8,
        pub Cancel: bool,
        pub CancelIrql: KIRQL,
        pub ApcEnvironment: i8,
        pub AllocationFlags: u8,
        pub UserIosb: *mut IO_STATUS_BLOCK,
        pub UserEvent: *mut KEVENT,
        pub UserApcRoutine: PIO_APC_ROUTINE,
        pub UserApcContext: *mut c_void,
        pub CancelRoutine: PDRIVER_CANCEL,
        pub UserBuffer: *mut c_void,
        pub Overlay: IRP_0,
    }
);

c_type!(
    pub struct IRP_0 {
        pub DriverContext: [*mut c_void; 4],
        pub Thread: *mut ETHREAD,
        pub AuxiliaryBuffer: *mut i8,
        pub ListEntry: LIST_ENTRY,
        pub CurrentStackLocation: *mut IO_STACK_LOCATION,
        pub PacketType: u32,
        pub OriginalFileObject: *mut FILE_OBJECT,
    }
);

// L31299
c_type!(
    // #[irql_requires_same]
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub type PIO_COMPLETION_ROUTINE =
        fn(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP, Context: *mut c_void) -> NTSTATUS;
);

pub const SL_INVOKE_ON_CANCEL: u8 = 0x20;
pub const SL_INVOKE_ON_SUCCESS: u8 = 0x40;
pub const SL_INVOKE_ON_ERROR: u8 = 0x80;

// L31643
c_type!(
    pub struct IO_STACK_LOCATION {
        pub MajorFunction: u8,
        pub MinorFunction: u8,
        pub Flags: u8,
        pub Control: u8,
        pub Parameters: IO_STACK_LOCATION_0,
        pub DeviceObject: *mut DEVICE_OBJECT,
        pub FileObject: *mut FILE_OBJECT,
        pub CompletionRoutine: PIO_COMPLETION_ROUTINE,
        pub Context: *mut c_void,
    }
);

c_type!(
    pub struct IO_STACK_LOCATION_0 {
        pub OutputBufferLength: u32,
        _padding0: u32,
        pub InputBufferLength: u32,
        _padding1: u32,
        pub IoControlCode: u32,
        pub Type3InputBuffer: *mut c_void,
    }
);

// L32458
extern "system" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IoCancelIrp(Irp: *mut IRP) -> bool;
}

// L32576
extern "fastcall" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IofCompleteRequest(Irp: *mut IRP, PriorityBoost: i8);
}

pub unsafe fn IoCompleteRequest(Irp: *mut IRP, PriorityBoost: i8) {
    IofCompleteRequest(Irp, PriorityBoost);
}

// L33379
pub unsafe fn IoGetCurrentIrpStackLocation(Irp: *mut IRP) -> *mut IO_STACK_LOCATION {
    debug_assert!((*Irp).CurrentLocation <= (*Irp).StackCount + 1);
    (*Irp).Overlay.CurrentStackLocation
}

// L33584
pub unsafe fn IoGetNextIrpStackLocation(Irp: *mut IRP) -> *mut IO_STACK_LOCATION {
    debug_assert!((*Irp).CurrentLocation > 0);
    (*Irp).Overlay.CurrentStackLocation.offset(-1)
}

// L33704
extern "system" {
    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IoInitializeIrp(Irp: *mut IRP, PacketSize: u16, StackSize: i8);

    // #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IoReuseIrp(Irp: *mut IRP, Iostatus: NTSTATUS);
}

// L33956
pub unsafe fn IoSetCompletionRoutine(
    Irp: *mut IRP,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    Context: *mut c_void,
    InvokeOnSuccess: bool,
    InvokeOnError: bool,
    InvokeOnCancel: bool,
) {
    debug_assert!(
        !(InvokeOnSuccess || InvokeOnError || InvokeOnCancel) || CompletionRoutine.is_some()
    );
    let irp_sp = IoGetNextIrpStackLocation(Irp);
    (*irp_sp).CompletionRoutine = CompletionRoutine;
    (*irp_sp).Context = Context;
    (*irp_sp).Control = 0;
    if InvokeOnSuccess {
        (*irp_sp).Control = SL_INVOKE_ON_SUCCESS;
    }
    if InvokeOnError {
        (*irp_sp).Control |= SL_INVOKE_ON_ERROR;
    }
    if InvokeOnCancel {
        (*irp_sp).Control |= SL_INVOKE_ON_CANCEL;
    }
}

// L37876
extern "system" {
    pub fn KeQueryPerformanceCounter(PerformanceFrequency: *mut LARGE_INTEGER) -> LARGE_INTEGER;
}

// L43369
extern "system" {
    // #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn ZwClose(Handle: HANDLE) -> NTSTATUS;
}
