use alloc::alloc::{GlobalAlloc, Layout};

use crate::windows::prelude as win;

struct NonPagedAllocator;

impl NonPagedAllocator {
    const GLOBAL_POOL_TAG: u32 = u32::from_ne_bytes([b'N', b'V', b'E', b'G']);
}

unsafe impl GlobalAlloc for NonPagedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        win::ExAllocatePool2(
            win::POOL_FLAGS::POOL_FLAG_NON_PAGED,
            layout.size(),
            Self::GLOBAL_POOL_TAG,
        )
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        win::ExFreePool(ptr)
    }
}

#[global_allocator]
static ALLOCATOR: NonPagedAllocator = NonPagedAllocator;
