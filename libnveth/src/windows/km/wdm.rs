use core::{mem, ptr, usize};

use sal::*;

use crate::windows::shared::{
    dpfilter::DPFLTR_TYPE,
    ntdef::{
        EVENT_TYPE, HANDLE, KIRQL, LARGE_INTEGER, LIST_ENTRY, NTSTATUS, OBJECT_ATTRIBUTES, PVOID,
    },
};

c_type!(
    pub struct EPROCESS;
);

c_type!(
    pub struct ETHREAD;
);

pub type KPRIORITY = i32;

c_type!(
    pub struct KSPIN_LOCK(usize);
);

c_type!(
    pub struct SECURITY_DESCRIPTOR;
);

c_type!(
    pub struct ACCESS_MASK(pub u32);
);

pub const SYNCHRONIZE: ACCESS_MASK = ACCESS_MASK(0x00100000);
pub const STANDARD_RIGHTS_REQUIRED: ACCESS_MASK = ACCESS_MASK(0x000F0000);

c_type!(
    pub struct IO_STATUS_BLOCK {
        pub status: NTSTATUS, // ...
        pub information: usize,
    }
);

c_type!(
    pub enum INTERFACE_TYPE {
        Internal = 0,
    }
);

pub const THREAD_ALL_ACCESS: ACCESS_MASK =
    ACCESS_MASK(STANDARD_RIGHTS_REQUIRED.0 | SYNCHRONIZE.0 | 0xFFFF);

c_type!(
    pub struct CLIENT_ID;
);

c_type!(
    #[repr(align(16))]
    pub struct SLIST_ENTRY {
        next: *mut SLIST_ENTRY,
    }
);

c_type!(
    #[repr(align(16))]
    pub struct SLIST_HEADER {
        header_x64: [u64; 2], // ...
    }
);

extern "C" {
    pub fn DbgPrintEx(component_id: DPFLTR_TYPE, level: u32, format: *const u8, ...) -> u32;
}

c_type!(
    pub type PALLOCATE_FUNCTION_EX = fn(
        pool_type: POOL_TYPE,
        number_of_bytes: usize,
        tag: u32,
        lookaside: *mut LOOKASIDE_LIST_EX,
    ) -> PVOID;
);

c_type!(
    pub type PFREE_FUNCTION_EX = fn(buffer: PVOID, lookaside: *mut LOOKASIDE_LIST_EX) -> ();
);

c_type!(
    pub struct GENERAL_LOOKASIDE_POOL {
        list_head: SLIST_HEADER, // ...
        depth: u16,
        maximum_depth: u16,
        total_allocates: u32,
        allocate_misses: u32, // ...
        total_frees: u32,
        free_misses: u32, // ...
        r#type: POOL_TYPE,
        tag: u32,
        size: u32,
        allocate_ex: PALLOCATE_FUNCTION_EX, // ...
        free_ex: PFREE_FUNCTION_EX,         // ...
        list_entry: LIST_ENTRY,
        last_total_allocates: u32,
        last_allocate_misses: u32, // ...
        future: [u32; 2],
    }
);

c_type!(
    #[repr(i8)]
    pub enum KPROCESSOR_MODE {
        KernelMode = 0,
    }
);

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SHIFT: usize = 12;

c_type!(
    pub struct MDL {
        pub next: *mut MDL, // NDIS_MDL_LINKAGE
        size: i16,
        mdl_flags: i16,
        process: *mut EPROCESS,
        mapped_system_va: PVOID,
        start_va: PVOID,
        byte_count: u32,
        byte_offset: u32,
    }
);

pub const MDL_MAPPED_TO_SYSTEM_VA: i16 = 0x0001;
pub const MDL_SOURCE_IS_NONPAGED_POOL: i16 = 0x0004;

c_type!(
    pub struct DISPATCHER_HEADER_u_s {
        pub r#ype: u8,
        pub signalling: u8,
        pub size: u8,
        pub reserved1: u8,
    }
);

c_type!(
    pub struct DISPATCHER_HEADER_u {
        pub s: DISPATCHER_HEADER_u_s,
        // ...
    }
);

c_type!(
    pub struct DISPATCHER_HEADER {
        pub u: DISPATCHER_HEADER_u,
        pub signal_state: i32,
        pub wait_list_head: LIST_ENTRY,
    }
);

c_type!(
    pub struct KEVENT {
        pub header: DISPATCHER_HEADER,
    }
);

pub type PFN_NUMBER = u64;

c_type!(
    pub enum KWAIT_REASON {
        Executive = 0,
    }
);

c_type!(
    pub type PKSTART_ROUTINE = fn(start_context: PVOID) -> ();
);

extern "system" {
    pub fn KeInitializeEvent(event: *mut KEVENT, r#type: EVENT_TYPE, state: bool) -> ();

    #[when(wait == false, irql_requires_max(DISPATCH_LEVEL))]
    #[when(wait == true, irql_requires_max(APC_LEVEL))]
    pub fn KeSetEvent(event: *mut KEVENT, increment: KPRIORITY, wait: bool) -> i32;

    #[when(timeout.is_null(), irql_requires_max(APC_LEVEL))]
    pub fn KeWaitForSingleObject(
        object: PVOID,
        wait_reason: KWAIT_REASON,
        wait_mode: KPROCESSOR_MODE,
        alertable: bool,
        timeout: *const LARGE_INTEGER,
    ) -> NTSTATUS;
}

c_type!(
    pub enum POOL_TYPE {
        NonPagedPool = 0,
        PagedPool = 1,
    }
);

c_type!(
    #[repr(u64)]
    pub enum POOL_FLAGS {
        POOL_FLAG_NON_PAGED = 0x0000000000000040,
        POOL_FLAG_PAGED = 0x0000000000000100,
    }
);

extern "system" {
    #[when((flags & POOL_FLAGS::POOL_FLAG_PAGED) != 0, irql_requires_max(APC_LEVEL))]
    #[when((flags & POOL_FLAGS::POOL_FLAG_PAGED) == 0, irql_requires_max(DISPATCH_LEVEL))]
    pub fn ExAllocatePool2(flags: POOL_FLAGS, number_of_bytes: usize, tag: u32) -> *mut u8;

    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn ExFreePool(p: *mut u8) -> ();

    pub fn InitializeSListHead(slist_head: *mut SLIST_HEADER) -> ();

    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn ExQueryDepthSList(slist_head: *const SLIST_HEADER) -> u16;
}

pub unsafe fn QueryDepthSList(slist_head: *const SLIST_HEADER) -> u16 {
    ExQueryDepthSList(slist_head)
}

// ExInitializeSListHead
// ExInterlockedPopEntrySList
// ExInterlockedPushEntrySList

pub unsafe fn InterlockedPopEntrySList(list_head: *mut SLIST_HEADER) -> *mut SLIST_ENTRY {
    ExpInterlockedPopEntrySList(list_head)
}

pub unsafe fn InterlockedPushEntrySList(
    list_head: *mut SLIST_HEADER,
    list_tail: *mut SLIST_ENTRY,
) -> *mut SLIST_ENTRY {
    ExpInterlockedPushEntrySList(list_head, list_tail)
}

extern "system" {
    pub fn ExpInterlockedPopEntrySList(list_head: *mut SLIST_HEADER) -> *mut SLIST_ENTRY;

    pub fn ExpInterlockedPushEntrySList(
        list_head: *mut SLIST_HEADER,
        list_tail: *mut SLIST_ENTRY,
    ) -> *mut SLIST_ENTRY;
}

c_type!(
    pub struct LOOKASIDE_LIST_EX {
        pub l: GENERAL_LOOKASIDE_POOL,
    }
);

extern "system" {
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn ExInitializeLookasideListEx(
        lookaside: *mut LOOKASIDE_LIST_EX,
        allocate: PALLOCATE_FUNCTION_EX,
        free: PFREE_FUNCTION_EX,
        pool_type: POOL_TYPE,
        flags: u32,
        size: usize,
        tag: u32,
        depth: u16,
    ) -> NTSTATUS;

    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn ExDeleteLookasideListEx(lookaside: *mut LOOKASIDE_LIST_EX) -> ();
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub fn ExAllocateFromLookasideListEx(lookaside: *mut LOOKASIDE_LIST_EX) -> PVOID {
    unsafe {
        let mut entry: PVOID;

        (*lookaside).l.total_allocates += 1;
        entry = InterlockedPopEntrySList(&mut (*lookaside).l.list_head) as _;
        if entry.is_null() {
            (*lookaside).l.allocate_misses += 1;
            entry = (*lookaside).l.allocate_ex.unwrap()(
                (*lookaside).l.r#type,
                (*lookaside).l.size as _,
                (*lookaside).l.tag,
                lookaside,
            );
        }

        entry
    }
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub fn ExFreeToLookasideListEx(lookaside: *mut LOOKASIDE_LIST_EX, entry: PVOID) {
    unsafe {
        (*lookaside).l.total_frees += 1;
        if QueryDepthSList(&mut (*lookaside).l.list_head) >= (*lookaside).l.depth {
            (*lookaside).l.free_misses += 1;
            (*lookaside).l.free_ex.unwrap()(entry, lookaside);
        } else {
            InterlockedPushEntrySList(&mut (*lookaside).l.list_head, entry as _);
        }
    }
}

c_type!(
    #[derive(Default)]
    pub struct EX_SPIN_LOCK(i32);
);

extern "system" {
    pub fn ExAcquireSpinLockShared(spin_lock: *mut EX_SPIN_LOCK) -> KIRQL;

    #[irql_requires(DISPATCH_LEVEL)]
    pub fn ExReleaseSpinLockShared(spin_lock: *mut EX_SPIN_LOCK, old_irql: KIRQL) -> ();

    pub fn ExAcquireSpinLockExclusive(spin_lock: *mut EX_SPIN_LOCK) -> KIRQL;

    #[irql_requires(DISPATCH_LEVEL)]
    pub fn ExReleaseSpinLockExclusive(spin_lock: *mut EX_SPIN_LOCK, old_irql: KIRQL) -> ();
}

pub const unsafe fn BYTE_OFFSET(va: PVOID) -> u32 {
    (va as usize & (PAGE_SIZE - 1)) as _
}

pub fn PAGE_ALIGN(va: PVOID) -> PVOID {
    (va as usize & !(PAGE_SIZE - 1)) as _
}

pub const unsafe fn ADDRESS_AND_SIZE_TO_SPAN_PAGES(va: PVOID, size: usize) -> usize {
    (BYTE_OFFSET(va) as usize + size + (PAGE_SIZE - 1)) >> PAGE_SHIFT
}

pub fn MmGetMdlByteCount(mdl: *const MDL) -> u32 {
    unsafe { (*mdl).byte_count }
}

extern "system" {
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn MmBuildMdlForNonPagedPool(memory_descriptor_list: *mut MDL) -> ();
}

pub type NODE_REQUIREMENT = u32;

pub const unsafe fn MmSizeOfMdl(base: PVOID, length: usize) -> usize {
    mem::size_of::<MDL>()
        + mem::size_of::<PFN_NUMBER>() * ADDRESS_AND_SIZE_TO_SPAN_PAGES(base, length)
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub unsafe fn MmInitializeMdl(
    memory_descriptor_list: *mut MDL,
    base_va: PVOID,
    length: usize,
) -> () {
    ptr::raw_mut!((*memory_descriptor_list).next).write(ptr::null_mut());
    ptr::raw_mut!((*memory_descriptor_list).size).write(MmSizeOfMdl(base_va, length) as _);
    ptr::raw_mut!((*memory_descriptor_list).mdl_flags).write(0);
    ptr::raw_mut!((*memory_descriptor_list).start_va).write(PAGE_ALIGN(base_va));
    ptr::raw_mut!((*memory_descriptor_list).byte_offset).write(BYTE_OFFSET(base_va));
    ptr::raw_mut!((*memory_descriptor_list).byte_count).write(length as _);
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub fn MmGetSystemAddressForMdlSafe(mdl: *mut MDL, _priority: u32) -> PVOID {
    unsafe {
        if (*mdl).mdl_flags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
            (*mdl).mapped_system_va
        } else {
            todo!()
        }
    }
}

extern "system" {
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn PsCreateSystemThread(
        thread_handle: *mut HANDLE,
        desired_access: ACCESS_MASK,
        object_attributes: *const OBJECT_ATTRIBUTES,
        process_handle: HANDLE,
        client_id: *mut CLIENT_ID,
        start_routine: PKSTART_ROUTINE,
        start_context: PVOID,
    ) -> NTSTATUS;

    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn PsTerminateSystemThread(exit_status: NTSTATUS) -> NTSTATUS;
}

c_type!(
    pub type PDRIVER_CANCEL = fn(device_object: *mut DEVICE_OBJECT, irp: *mut IRP) -> ();
);

c_type!(
    pub struct DEVICE_OBJECT;
);

c_type!(
    pub struct DRIVER_OBJECT;
);

c_type!(
    pub struct FILE_OBJECT;
);

c_type!(
    pub union IRP_u {
        pub system_buffer: PVOID,
        // ...
    }
);

c_type!(
    pub struct IRP_u2_s {
        pub driver_context: [PVOID; 4], // ...
        pub thread: *mut ETHREAD,
        pub auxiliary_buffer: *mut i8,
        pub list_entry: LIST_ENTRY,
        pub current_stack_location: *mut IO_STACK_LOCATION,
        // ...
    }
);

c_type!(
    pub union IRP_u2 {
        pub overlay: IRP_u2_s,
        // ...
    }
);

c_type!(
    pub struct IRP {
        pub r#type: i16,
        pub size: u16,
        pub mdl_address: *mut MDL,
        pub flags: u32,
        pub associated_irp: IRP_u,
        pub thread_list_entry: LIST_ENTRY,
        pub io_status: IO_STATUS_BLOCK,
        pub requestor_mode: KPROCESSOR_MODE,
        pub pending_returned: bool,
        pub stack_count: i8,
        pub current_location: i8,
        pub cancel: bool,
        pub cancel_irql: KIRQL,
        pub apc_environment: i8,
        pub allocation_flags: u8,
        pub user_iosb: *mut IO_STATUS_BLOCK,
        pub user_event: *mut KEVENT,
        pub overlay: [PVOID; 2], // ...
        pub cancel_routine: PDRIVER_CANCEL,
        pub user_buffer: PVOID,
        pub tail: IRP_u2,
    }
);

c_type!(
    pub type PIO_COMPLETION_ROUTINE =
        fn(device_object: *const DEVICE_OBJECT, irp: *const IRP, context: PVOID) -> NTSTATUS;
);

pub const SL_INVOKE_ON_CANCEL: u8 = 0x20;
pub const SL_INVOKE_ON_SUCCESS: u8 = 0x40;
pub const SL_INVOKE_ON_ERROR: u8 = 0x80;

c_type!(
    pub struct IO_STACK_LOCATION_u_s {
        pub output_buffer_length: u32,
        _padding1: u32,
        pub input_buffer_length: u32,
        _padding2: u32,
        pub io_control_code: u32,
        _padding3: u32,
        pub type3_input_buffer: PVOID,
    }
);

c_type!(
    pub union IO_STACK_LOCATION_u {
        pub device_io_control: IO_STACK_LOCATION_u_s,
        // ...
    }
);

c_type!(
    pub struct IO_STACK_LOCATION {
        pub major_function: u8,
        pub minor_function: u8,
        pub flags: u8,
        pub control: u8,
        pub parameters: IO_STACK_LOCATION_u,
        pub device_object: *mut DEVICE_OBJECT,
        pub file_object: *mut FILE_OBJECT,
        pub completion_routine: PIO_COMPLETION_ROUTINE,
        pub context: PVOID,
    }
);

extern "system" {
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IoAllocateIrp(stack_size: i8, charge_quota: bool) -> *mut IRP;

    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IoCancelIrp(irp: *const IRP) -> bool;
}

extern "system" {
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IoFreeIrp(irp: *const IRP) -> ();
}

pub unsafe fn IoGetNextIrpStackLocation(irp: *const IRP) -> *mut IO_STACK_LOCATION {
    (*irp).tail.overlay.current_stack_location.offset(-1)
}

extern "system" {
    #[irql_requires_max(DISPATCH_LEVEL)]
    pub fn IoReuseIrp(irp: *mut IRP, iostatus: NTSTATUS) -> ();
}

#[irql_requires_max(DISPATCH_LEVEL)]
pub unsafe fn IoSetCompletionRoutine(
    irp: *const IRP,
    completion_routine: PIO_COMPLETION_ROUTINE,
    context: PVOID,
    invoke_on_success: bool,
    invoke_on_error: bool,
    invoke_on_cancel: bool,
) -> () {
    let irp_sp = IoGetNextIrpStackLocation(irp);
    ptr::raw_mut!((*irp_sp).completion_routine).write(completion_routine);
    ptr::raw_mut!((*irp_sp).context).write(context);
    ptr::raw_mut!((*irp_sp).control).write(0);

    if invoke_on_success {
        (*irp_sp).control = SL_INVOKE_ON_SUCCESS;
    }

    if invoke_on_error {
        (*irp_sp).control |= SL_INVOKE_ON_ERROR;
    }

    if invoke_on_cancel {
        (*irp_sp).control |= SL_INVOKE_ON_CANCEL;
    }
}

extern "system" {
    #[irql_requires_max(PASSIVE_LEVEL)]
    pub fn ZwClose(handle: HANDLE) -> NTSTATUS;
}
