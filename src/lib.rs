//! [`Buddies`](crate): A buddies allocator.
//!
//! This can be used to allocate blocks of different sizes from a single
//! fixed-width block, and is useful in bare-metal physical memory allocation
//! (which is how Linux does things).

#![no_std]

extern crate bitvec;
use bitvec::prelude::*;

/// [`RawBuddies`]: A slightly unsafe buddy allocator.
///
/// A small size and no standard library dependency is traded for an unsafe
/// structure. A safe shell can be constructed around this for built-in
/// allocation of resources as well as a safe allocation result.
pub struct RawBuddies<T> {
    /// The number of buddies.
    num: usize,
    /// A pointer to the first data element (size 2^num).
    data: *mut T,
    /// A pointer to the first bitspace byte.
    bits: *mut u8,
}

impl<T> RawBuddies<T> {
    /// Creates a new [`RawBuddies`].
    ///
    /// ### Conditions
    /// `data` and `bits` are not dropped as long as the instantiation lives.  
    /// `data` is at least of length `2^num`. It may be uninitialized.  
    /// `bits` is at least of length `2^num/8` (i.e it holds `2^num` bits). It
    /// must only contain `0`s (i.e `false`s).
    pub unsafe fn new(num: usize, data: *mut T, bits: *mut u8) -> Self {
        Self {
            num,
            data,
            bits,
        }
    }

    /// Checks if a block of size `2^n` `T`s can be allocated.
    ///
    /// ### Panics
    /// Panics if the block size is too large (`>= buddies`).
    pub fn can_allocate(&self, n: usize) -> bool {
        // Check if matching buddy contains free zones
        self.buddymap_ref(n).any()
    }

    /// Allocates a block of size `2^n` `T`s.
    ///
    /// Note for safe shells: You want to convert the reference to a pointer so
    /// that multiple allocations can exist simultaneously (or work around
    /// using `Rc` somehow).
    ///
    /// Returns the reference as well as the block index (for freeing later).
    ///
    /// ### Panics
    /// Panics if the block size is too large (`>= buddies`).
    pub fn allocate(&mut self, n: usize) -> Option<(&mut T, usize)> {
        // Find a free zone for the buddy (auto-exit on fail)
        let pos = self.buddymap_ref(n).iter().position(|s| s)?;
        // Mark it, and all above, as mutable
        for i in 0 .. (self.num - n) {
            let map = self.buddymap_mut(n + i);
            if map[pos >> i] {
                break; // This one is set; all above will be set too
            } else {
                map.set(pos >> i, true);
            }
        }
        // Return the block
        Some((unsafe { &mut *self.data.add(pos << n) }, pos))
    }

    /// Frees a given block by index and size.
    ///
    /// ### Panics
    /// Panics if the block size is too large (`>= buddies`).  
    /// Panics if the index is too large (`>= 2^(buddies-size-1)`).  
    /// Panics if the block was already free (possible double-free).  
    pub fn free(&mut self, n: usize, pos: usize) {
        assert!(n < self.num);
        assert!(pos < (1usize << (self.num - n - 1)));
        assert!(self.buddymap_ref(n)[pos]);

        unsafe { core::ptr::drop_in_place(self.data.add(pos << n)); }

        for i in 0 .. (self.num - n) {
            let map = self.buddymap_mut(n + i);
            map.set(pos >> i, false);
            // If the other one (in set for above) is non-empty, then stop
            if map[(pos >> i) ^ 1usize] {
                break;
            }
        }
    }

    /// Retrieves a bit slice for a certain buddy immutably.
    ///
    /// Primarily defined for [`can_allocate`].
    fn buddymap_ref(&self, n: usize) -> &BitSlice {
        assert!(n < self.num);
        // Index is 2^(num-n) from end
        let bits: &BitSlice = unsafe {
            core::slice::from_raw_parts(self.bits, 1usize << self.num)
        }.into();
        &bits[
            (1usize << self.num) - (1usize << (self.num - n))
         .. (1usize << self.num) - (1usize << (self.num - n - 1))
        ]
    }

    /// Retrieves a bit slice for a certain buddy mutably.
    fn buddymap_mut(&mut self, n: usize) -> &mut BitSlice {
        assert!(n < self.num);
        // Index is 2^(num-n) from end
        let bits: &mut BitSlice = unsafe {
            core::slice::from_raw_parts_mut(self.bits, 1usize << self.num)
        }.into();
        &mut bits[
            (1usize << self.num) - (1usize << (self.num - n))
         .. (1usize << self.num) - (1usize << (self.num - n - 1))
        ]
    }
}
