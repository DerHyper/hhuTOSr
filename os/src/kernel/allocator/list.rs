/* ╔═════════════════════════════════════════════════════════════════════════╗
 *  ║ Module: list                                                            ║
 *  ╟─────────────────────────────────────────────────────────────────────────╢
 *  ║ Descr.: Implementing a list heap allocator.                             ║
 *  ╟─────────────────────────────────────────────────────────────────────────╢
 *  ║ Author: Philipp Oppermann                                               ║
 *  ║         https://os.phil-opp.com/allocator-designs/                      ║
 *  ╚═════════════════════════════════════════════════════════════════════════╝
 */
use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
use crate::devices::kprint;
use crate::kernel::allocator::bump::BumpAllocator;
use crate::kernel::cpu as cpu;

/// Header of a free block in the list allocator.
struct ListNode {
    /// Size of the memory block
    size: usize,

    /// &'static mut type semantically describes an owned object behind
    /// a pointer. Basically, it’s a Box without a destructor that frees
    /// the object at the end of the scope. Its lifetime is static,
    /// meaning it will live for the entire duration of the program.
    /// Of course, this is not true in reality, as we might delete the
    /// list node at some point. But the compiler does not know this.
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// Creates a new ListNode with the given size and no next node.
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    /// Get the start address of the memory block.
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    /// Get the end address of the memory block.
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// A linked list allocator that uses a free list to manage memory.
pub struct LinkedListAllocator {
    head: ListNode,
    heap_start: usize,
    heap_end: usize,
}

impl LinkedListAllocator {
    /// Create a new empty linked list allocator.
    pub const fn new(heap_start: usize, heap_size: usize) -> LinkedListAllocator {
        LinkedListAllocator {
            head: ListNode::new(heap_size),
            heap_start,
            heap_end: heap_start + heap_size,
        }
    }

    /// Initialize the allocator with the heap bounds given in the constructor.
    pub unsafe fn init(&mut self) {
        kprintln!("list-allocator: init");

        // Add empty head
        let head = ListNode::new(0);
        self.head = head;

        // Add one single free block to the list that covers the whole heap.
        let size = self.heap_end - self.heap_start;
        let empty_node = self.heap_start as *mut ListNode;
        unsafe {
            (*empty_node).size = size;
            (*empty_node).next = None;
        }
        
        self.head.next = Some(unsafe{&mut (*empty_node)});
        //unsafe {self.add_free_block(self.heap_start, size)}
        kprintln!("list-allocator: init done");
    }

    /// Adds the given free memory block 'addr' to the front of the free list.
    unsafe fn add_free_block(&mut self, addr: usize, size: usize) {
        // Current_Head ist an stelle 0x5... und new_note_addr ist auch an stelle 0x5... 
        // Dadurch wird das Original verändert und eine REferenz auf sich selbst gespeichert
        kprintln!("list-allocator: add_free_block: addr=0x{:x}, size={}", addr, size);

        // Get current head
        let current_head = self.head.next.take();

        // Create new node
        let new_note_addr: *mut ListNode = addr as *mut ListNode;
        unsafe {
            (*new_note_addr).size = size;
            (*new_note_addr).next = current_head;
        }

        // Set new head
        self.head.next = Some(unsafe{ &mut *new_note_addr });
    }

    /// Search a free block with the given size and alignment and remove it from the list.
    fn find_free_block(&mut self, size: usize, align: usize) -> Option<&'static mut ListNode> {
        kprintln!("list-allocator: find_free_block: size={}, align={}", size, align);
        // Get the head of the list
        let mut current = &mut self.head.next; // mutable borrow
        
        // Iterate over the list and find a free block
        while let Some(node) = current.take() { // Take the current node
            kprintln!("   ... checking block: addr=0x{:x}, size={}", node.start_addr(), node.size);
            if LinkedListAllocator::check_block_for_alloc(node, size, align).is_ok() {
                // Remove node from the list
                kprintln!("   found free block: addr=0x{:x}, size={}", node.start_addr(), node.size);
                *current = node.next.take(); // Override current with next
                return Some(node);
            } else {
                let next_ptr: *mut Option<&'static mut ListNode> = &mut node.next; // Get next
                *current = Some(node); // Put the node back into the list

                unsafe {
                    current = &mut *next_ptr; // Set current to next
                }
            }
        }
        kprintln!("   found no free block");
        None
    }

    /// Check if the given block is large enough for an allocation with `size` and `align`.
    fn check_block_for_alloc(block: &ListNode, size: usize, align: usize) -> Result<(),()> {
        kprintln!("list-allocator: check_block_for_alloc: size={}, align={}", size, align);

        // Check if the block is aligned
        let start = block.start_addr();
        let aligned_start = align_up(start, align);
        if start != aligned_start {
            return Err(());
        }

        // Check if the block is large enough
        if block.size < size {
            return Err(());
        }

        Ok(())
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// block is also capable of storing a `ListNode`.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
        .align_to(align_of::<ListNode>())
        .expect("adjusting alignment failed")
        .pad_to_align();
        let size = layout.size().max(size_of::<ListNode>());

        (size, layout.align())
    }

    /// Dump the free list for debugging purposes.
    pub fn dump_free_list(&mut self) {
        println!("Dumping free memory list:");
        println!("   Heap start:   0x{:x}, heap end:   0x{:x}", self.heap_start, self.heap_end);

        //for node in self.head.next.iter() {
            //println!("   Block start:  0x{:x}, block end:  0x{:x}, block size: {}", node.start_addr(), node.end_addr(), node.size);
        //}

        // Get the head of the list
        let mut current = &mut self.head.next; // mutable borrow
        // Itter
        while let Some(node) = current.take() { // Take the current node
            let next_ptr: *mut Option<&'static mut ListNode> = &mut node.next; // Get next
            println!("   Block start:  0x{:x}, block end:  0x{:x}, block size: {}", node.start_addr(), node.end_addr(), node.size);
            unsafe {
                current = &mut *next_ptr; // Set current to next
            }
        }
        
        println!("");

        unsafe {self.init()};

    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        kprintln!("list-alloc: size={}, align={}", layout.size(), layout.align());
        let (size, align) = LinkedListAllocator::size_align(layout);

        // check for block
        let block_link = self.find_free_block(size, align);
        let block = match block_link { Some(v) => v, none => {
            kprintln!("   no free block found");
            return ptr::null_mut();
        }};

        // If remaining memory is big enough, split the block
        let memory_remaining = block.size - size;
        if memory_remaining > mem::size_of::<ListNode>() {
            let new_block_addr = block.start_addr() + size;
            unsafe {self.add_free_block(new_block_addr, memory_remaining);}
            block.size = size;
        }
        
        // return pointer to the allocated memory
        block.start_addr() as *mut u8
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        kprintln!("list-dealloc: size={}, align={}", layout.size(), layout.align());

        let (size, _) = LinkedListAllocator::size_align(layout);

        unsafe {
            self.add_free_block(ptr as usize, size)
        }
    }

}

// Trait required by the Rust runtime for heap allocations
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
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
