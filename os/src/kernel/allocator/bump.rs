/* ╔═════════════════════════════════════════════════════════════════════════╗
 *   ║ Module: bump                                                            ║
 *   ╟─────────────────────────────────────────────────────────────────────────╢
 *   ║ Descr.: Implementing a basic heap allocator which cannot use            ║
 *   ║         deallocated memory. Thus it is only for learning and testing.   ║
 *   ╟─────────────────────────────────────────────────────────────────────────╢
 *   ║ Author: Philipp Oppermann                                               ║
 *   ║         https://os.phil-opp.com/allocator-designs/                      ║
 *   ╚═════════════════════════════════════════════════════════════════════════╝
 */
use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// A simple bump allocator that allocates memory in a linear fashion.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Create a new empty bump allocator.
    pub const fn new(heap_start: usize, heap_size: usize) -> BumpAllocator {
        BumpAllocator {
            heap_start,
            heap_end: heap_start + heap_size,
            next: heap_start,
            allocations: 0,
        }
    }

    /// Initialize the bump allocator.
    /// No-op for this allocator, but required by the kernel.
    pub unsafe fn init(&mut self) {}

    /// Dump free memory for debugging purposes.
    pub fn dump_free_list(&mut self) {
        println!("Dumping free memory list:");
        println!("   Heap start:   0x{:x}, heap end:   0x{:x}", self.heap_start, self.heap_end);
        println!("   Block start:  0x{:x}, block end:  0x{:x}, block size: {}\n", self.next, self.heap_end, self.heap_end - self.next);
        
        // Reset next pointer
        self.next = self.heap_start;
        self.allocations = 0;

        

    }

    /// Allocate memory of the given size and alignment.
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {

        let size = layout.size();
        let align = layout.align();
        let aligned_next = align_up(self.next, align);

        // Check if there is enough space
        if aligned_next + size > self.heap_end {
            return ptr::null_mut(); // Not enough space
        }

        // Update next pointer
        self.next = aligned_next + size;
        self.allocations += 1;

        // Return Pointer
        let next_ptr: *mut u8 = aligned_next as *mut u8;
        next_ptr
    }

    /// Deallocate memory (not supported by bump allocator).
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {}
}

// Trait required by the Rust runtime for heap allocations
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            self.lock().alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.lock().dealloc(ptr, layout);
        }
    }
}
