use crate::alloc::{GlobalAlloc, Layout, System};

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        panic!("no default allocator")
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, _layout: Layout) -> *mut u8 {
        panic!("no default allocator")
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("no default allocator")
    }

    #[inline]
    unsafe fn realloc(&self, _ptr: *mut u8, _layout: Layout, _new_size: usize) -> *mut u8 {
        panic!("no default allocator")
    }
}
