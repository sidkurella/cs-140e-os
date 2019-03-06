use std::fmt;
use std::mem;
use std::mem::ManuallyDrop;
use std::ptr;
use alloc::alloc::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

const PAGE_ORDER : usize = 14;
const PAGE_SIZE : usize = 1 << PAGE_ORDER;

const MAX_BLOCK_ORDER : usize = 10;

use console::{kprint, kprintln, CONSOLE};

macro_rules! construct_array {
    ($e: expr, $n:expr) => (
        {
            use std::mem;
            use std::ptr;

            struct ArrayBuilder<T> {
                len: isize,
                data: *mut T,
            }

            impl<T> ArrayBuilder<T> {
                fn new(v: &mut ManuallyDrop<[T; $n]>) -> ArrayBuilder<T> {
                    ArrayBuilder {
                        len: 0,
                        data: (&mut *v as *mut _) as *mut _
                    }
                }

                unsafe fn write(&mut self, val: T) {
                    debug_assert!(self.len < $n as isize);

                    ptr::write(self.data.offset(self.len), val);
                    self.len += 1;
                }
            }

            impl<T> Drop for ArrayBuilder<T> {
                fn drop(&mut self) {
                    unsafe {
                        while self.len > 0 {
                            let offset = self.len - 1;
                            self.len -= 1;
                            ptr::drop_in_place(self.data.offset(offset));
                        }
                    }
                }
            }

            let mut v: ManuallyDrop<[_; $n]> = ManuallyDrop::new(unsafe {
                mem::uninitialized()
            });

            if false { v[0] = $e(0); }

            let mut builder = ArrayBuilder::new(&mut v);
            for i in 0..$n {
                let mut val = $e(i);
                unsafe { builder.write(val); }
            }

            builder.len = 0;
            ManuallyDrop::into_inner(v)
        }
    )
}


#[derive(Debug)]
struct Bitmap {
    length: usize,
    data: usize
}

impl Bitmap {
    pub unsafe fn new(length: usize, data: *mut u8) -> Bitmap {
        Bitmap {
            length: length,
            data: data as usize
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }
    pub fn length_bytes(&self) -> usize {
        (self.length + 7) / 8
    }

    #[inline]
    fn slice(&self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data as *mut u8, self.length_bytes())
        }
    }

    pub fn zero(&self) {
        for byte in self.slice().iter_mut() {
            *byte = 0;
        }
    }


    #[inline]
    fn byte(&self, bit: usize) -> &mut u8 {
        &mut self.slice()[bit / 8]
    }

    pub fn get_bit(&self, bit: usize) -> bool {
        assert!(bit <= self.length);

        ((*self.byte(bit) >> (bit % 8)) & 1) != 0
    }

    pub fn clear_bit(&self, bit: usize) {
        assert!(bit <= self.length);

        *self.byte(bit) &= !(1 << (bit % 8));
    }

    pub fn set_bit(&self, bit: usize) {
        assert!(bit <= self.length);

        *self.byte(bit) |= 1 << (bit % 8);
    }

    pub fn toggle_bit(&self, bit: usize) {
        assert!(bit <= self.length);

        *self.byte(bit) ^= 1 << (bit % 8);
    }
}


#[derive(Debug)]
struct BuddyBlockAllocatorZone {
    free_list: LinkedList,
    start: usize,
    order: usize,
    map: Bitmap
}

impl BuddyBlockAllocatorZone {
    pub fn new(start: usize, order: usize, map: Bitmap) -> BuddyBlockAllocatorZone {
        map.zero();
        BuddyBlockAllocatorZone {
            free_list: LinkedList::new(),
            start: start,
            order: order,
            map: map
        }
    }

    #[inline]
    fn is_valid_ptr(&self, ptr: *mut u8) -> bool {
        (ptr as usize - self.start) & ((PAGE_SIZE << self.order) - 1) == 0
    }

    #[inline]
    fn get_ptr_index(&self, ptr: *mut u8) -> usize {
        (ptr as usize - self.start) >> (PAGE_ORDER + self.order + 1)
    }

    #[inline]
    pub fn get_ptr_buddy(&self, ptr: *mut u8) -> *mut u8 {
        let mask = 1 << (PAGE_ORDER + self.order);

        (((ptr as usize - self.start) ^ mask) + self.start) as *mut u8
    }

    pub fn alloc(&mut self) -> Result<*mut u8, AllocErr> {
        match self.free_list.pop() {
            Some(ptr) => {
                let index = self.get_ptr_index(ptr);
                self.map.toggle_bit(index);
                Ok(ptr as *mut u8)
            },
            None => Err(AllocErr)
        }
    }

    pub fn free(&mut self, ptr: *mut u8) -> Option<*mut u8> {
        assert!(self.is_valid_ptr(ptr));

        let index = self.get_ptr_index(ptr);

        self.map.toggle_bit(index);
        if self.order == MAX_BLOCK_ORDER || self.map.get_bit(index) {
            unsafe { self.free_list.push(ptr); }

            None
        } else {
            let buddy = self.get_ptr_buddy(ptr);

            unsafe { self.free_list.remove(buddy); }

            if (buddy as usize) < (ptr as usize) {
                Some(buddy)
            } else {
                Some(ptr)
            }
        }
    }
}

#[derive(Debug)]
struct BuddyBlockAllocator {
    zones: [BuddyBlockAllocatorZone; MAX_BLOCK_ORDER + 1]
}


impl BuddyBlockAllocator {
    pub fn new(start: usize, end: usize) -> BuddyBlockAllocator {
        let num_pages = (end - start) >> PAGE_ORDER;

        let mut arr_ptr = start;
        let mut maps = construct_array!(|idx| {
            let bitmap = unsafe { Bitmap::new(num_pages >> (idx + 1), arr_ptr as *mut u8) };
            arr_ptr += bitmap.length_bytes();
            Some(bitmap)
        }, MAX_BLOCK_ORDER + 1);

        let mut mem_start = align_up(arr_ptr, PAGE_SIZE);
        let mem_end = align_down(end, PAGE_SIZE);

        assert!(mem_start <= mem_end, "not enough available memory");

        let mut allocator = BuddyBlockAllocator {
            zones: construct_array!(|idx| {
                    BuddyBlockAllocatorZone::new(mem_start, idx,
                                                 maps[idx].take().unwrap())
                }, MAX_BLOCK_ORDER + 1)
        };

        for order in (0 ..= MAX_BLOCK_ORDER).rev() {
            let chunk_bytes = PAGE_SIZE << order;

            loop {
                let mem_next = mem_start.saturating_add(chunk_bytes);
                if mem_next <= mem_end {
                    allocator.free(mem_start as *mut u8, order);
                    mem_start = mem_next;
                } else {
                    break;
                }
            }
        }

        allocator
    }

    pub fn alloc(&mut self, order: usize) -> Result<*mut u8, AllocErr> {
        assert!(order <= MAX_BLOCK_ORDER);

        if let Ok(ptr) = self.zones[order].alloc() {
            Ok(ptr)
        } else if (order != MAX_BLOCK_ORDER) {
            if let Ok(lower) = self.alloc(order + 1) {
                let higher = self.zones[order].get_ptr_buddy(lower);

                self.zones[order].free(lower);

                Ok(higher)
            } else {
                Err(AllocErr)
            }
        } else {
            Err(AllocErr)
        }
    }

    pub fn free(&mut self, ptr: *mut u8, order: usize) {
        if let Some(coalesce) = self.zones[order].free(ptr) {
            if order != MAX_BLOCK_ORDER {
                self.free(coalesce, order + 1);
            }
        }
    }
}


/// A simple allocator that allocates based on size classes.
#[derive(Debug)]
pub struct Allocator {
    inner: BuddyBlockAllocator
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            inner: BuddyBlockAllocator::new(start, end)
        }
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        assert!(layout.align() <= PAGE_SIZE);

        let mut order = PAGE_ORDER;
        while (1 << order) < layout.size() { order += 1; }

        self.inner.alloc(order - PAGE_ORDER)
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        assert!(layout.align() <= PAGE_SIZE);

        let mut order = PAGE_ORDER;
        while (1 << order) < layout.size() { order += 1; }

        self.inner.free(ptr, order - PAGE_ORDER);
    }
}
