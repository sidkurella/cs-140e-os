#![allow(dead_code)]

use std::{fmt, ptr};

/// An _instrusive_ linked list of addresses.
///
/// A `LinkedList` maintains a list of `*mut u8`s. The user of the
/// `LinkedList` guarantees that the passed in pointer refers to valid, unique,
/// writeable memory at least `usize` in size.
///
/// # Usage
///
/// A list is created using `LinkedList::new()`. A new address can be prepended
/// using `push()`. The first address in the list, if any, can be removed and
/// returned using `pop()` or returned (but not removed) using `peek()`.
///
/// ```rust
/// # let address_1 = (&mut (1 as usize)) as *mut u8;
/// # let address_2 = (&mut (2 as usize)) as *mut u8;
/// let mut list = LinkedList::new();
/// unsafe {
///     list.push(address_1);
///     list.push(address_2);
/// }
///
/// assert_eq!(list.peek(), Some(address_2));
/// assert_eq!(list.pop(), Some(address_2));
/// assert_eq!(list.pop(), Some(address_1));
/// assert_eq!(list.pop(), None);
/// ```
///
/// `LinkedList` exposes two iterators. The first, obtained via `iter()`,
/// iterates over all of the addresses in the list. The second, returned from
/// `iter_mut()`, returns `Node`s that refer to each address in the list. The
/// `value()` and `pop()` methods of `Node` can be used to read the value or pop
/// the value from the list, respectively.
///
/// ```rust
/// # let address_1 = (&mut (1 as usize)) as *mut u8;
/// # let address_2 = (&mut (2 as usize)) as *mut u8;
/// # let address_3 = (&mut (3 as usize)) as *mut u8;
/// let mut list = LinkedList::new();
/// unsafe {
///     list.push(address_1);
///     list.push(address_2);
///     list.push(address_3);
/// }
///
/// for node in list.iter_mut() {
///     if node.value() == address_2 {
///         node.pop();
///     }
/// }
///
/// assert_eq!(list.pop(), Some(address_3));
/// assert_eq!(list.pop(), Some(address_1));
/// assert_eq!(list.pop(), None);
/// ```

#[repr(C)]
struct LinkedListEntry {
    prev: *mut LinkedListEntry,
    next: *mut LinkedListEntry
}

impl LinkedListEntry {

    pub unsafe fn prev(&self) -> Option<&mut LinkedListEntry> {
        self.prev.as_mut()
    }

    pub unsafe fn next(&self) -> Option<&mut LinkedListEntry> {
        self.next.as_mut()
    }
}

#[derive(Copy, Clone)]
pub struct LinkedList {
    head: *mut LinkedListEntry,
    tail: *mut LinkedListEntry
}

unsafe impl Send for LinkedList {}

impl LinkedList {
    /// Returns a new, empty linked list.
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),
            tail: ptr::null_mut()
        }
    }

    /// Returns `true` if the list is empty and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    /// Pushes the address `item` to the front of the list.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` refers to unique, writeable memory at
    /// least `usize` in size that is valid as long as `item` resides in `self`.
    /// Barring the uniqueness constraint, this is equivalent to ensuring that
    /// `*item = some_usize` is a safe operation as long as the pointer resides
    /// in `self`.
    pub unsafe fn push(&mut self, item: *mut u8) {
        let item_raw = item as *mut LinkedListEntry;
        let item_ll  = &mut *item_raw;

        item_ll.prev = ptr::null_mut();
        item_ll.next = self.head;

        self.head = item_raw;
        if self.tail == ptr::null_mut() {
            self.tail = item_raw;
        }
    }

    /// Removes and returns the first item in the list, if any.
    pub fn pop(&mut self) -> Option<*mut u8> {
        let item_raw = self.peek()? as *mut LinkedListEntry;
        let item_ll  = unsafe { &mut *item_raw };

        self.head = item_ll.next;

        if let Some(next_ll) = unsafe { item_ll.next() } {
            next_ll.prev = ptr::null_mut();
        } else {
            self.tail = ptr::null_mut();
        }

        Some(item_raw as *mut u8)
    }

    /// Returns the first item in the list without removing it, if any.
    pub fn peek(&self) -> Option<*mut u8> {
        match self.is_empty() {
            true => None,
            false => Some(self.head as *mut u8),
        }
    }


    /// Removes the specified item from the list.
    pub unsafe fn remove(&mut self, item: *mut u8) {
        let item_raw = item as *mut LinkedListEntry;
        let item_ll  = &mut *item_raw;

        if let Some(prev_ll) = item_ll.prev() {
            prev_ll.next = item_ll.next;
        } else {
            self.head = ptr::null_mut();
        }

        if let Some(next_ll) = item_ll.next() {
            next_ll.prev = item_ll.prev;
        } else {
            self.tail = ptr::null_mut();
        }
    }

    /// Returns an iterator over the items in this list.
    pub fn iter(&self) -> Iter {
        Iter { current: self.head, _list: self }
    }
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// An iterator over the items of the linked list.
pub struct Iter<'a> {
    _list: &'a LinkedList,
    current: *mut LinkedListEntry
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut u8;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.current;
        self.current = unsafe { (*self.current).next };
        Some(value as *mut u8)
    }
}
