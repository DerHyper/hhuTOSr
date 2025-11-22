use core::fmt;
use core::ops::{Add, Sub};
use crate::consts::PAGE_FRAME_SIZE;
use crate::library::input::getch;
use crate::library::spinlock::Spinlock as Mutex;

pub static FRAME_ALLOCATOR: Mutex<PfListAllocator> = Mutex::new(PfListAllocator::new());

/// Represents a physical address in memory and allows accessing it via pointers.
/// Basic arithmetic operations are implemented for easy address manipulation.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Phys(0x{:016x})", self.0)
    }
}

impl From<PhysAddr> for u64 {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl Add<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_add(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_sub(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_add(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_sub(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_add(rhs).unwrap();
        PhysAddr(res)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_sub(rhs).unwrap();
        PhysAddr(res)
    }
}

/// A node in the physical frame free list.
/// Contains the size of the free block and a pointer to the next node.
struct PfListNode {
    size: usize,
    next: Option<&'static mut PfListNode>
}

impl PfListNode {
    const fn new(size: usize) -> Self {
        PfListNode { size, next: None }
    }

    fn start_addr(&self) -> PhysAddr {
        PhysAddr::new(self as *const Self as u64)
    }

    fn end_addr(&self) -> PhysAddr {
        self.start_addr() + self.size
    }
}

/// A physical frame allocator that uses a linked list to manage free memory blocks.
/// Memory blocks are always aligned to PAGE_FRAME_SIZE (4096 bytes).
pub struct PfListAllocator {
    head: PfListNode
}

impl PfListAllocator {
    /// Create a new empty physical frame list allocator.
    pub const fn new() -> PfListAllocator {
        PfListAllocator {
            head: PfListNode::new(0)
        }
    }

    /// Try to allocate a block of 'num_frames' physical frames.
    /// Returns the starting physical address of the allocated block on success.
    /// The found block is filled with zeroes. If the block is larger than requested,
    /// the remaining part is added back to the free list.
    /// If no suitable block is found, returns None.
    pub unsafe fn alloc_block(&mut self, num_frames: usize) -> Option<PhysAddr> {
        
        let mut current_block = &mut self.head.next;
        let search_size = PAGE_FRAME_SIZE*num_frames;

        // Iterate over the list and find a free block
        while let Some(node) = current_block.take() {

            // Found free block
            if node.size == search_size {
                
                let addr = node.start_addr();
                *current_block = node.next.take();
                fill_block_with_zeros(node);
                return Some(addr);
            
            // Found oversized free block
            } else if node.size > search_size {
                let addr = node.start_addr();
                *current_block = node.next.take();
                
                let new_block_num_frames = (node.size - search_size)/PAGE_FRAME_SIZE;
                let new_block_addr = node.start_addr() + search_size;
                unsafe {
                    self.free_block(new_block_addr, new_block_num_frames);   
                }
                
                fill_block_with_zeros(node);
                return Some(addr);

            // Found no free block
            } else {
                let next_ptr: *mut Option<&'static mut PfListNode> = &mut node.next; // Get next
                *current_block = Some(node); // Put the node back into the list

                unsafe {
                    current_block = &mut *next_ptr; // Set current to next
                }
            }
        }
        
        // No free block found
        Option::None
    }

    /// Free a previously allocated block of 'num_frames' physical frames starting at 'addr'.
    /// The address must be aligned to PAGE_FRAME_SIZE (4096 bytes).
    /// The freed block is merged with adjacent free blocks if possible.
    pub unsafe fn free_block(&mut self, addr_start: PhysAddr, num_frames: usize) {
        
        let mut current_block = &mut self.head.next;
        let size =  PAGE_FRAME_SIZE*num_frames;
        let addr_end = addr_start + PAGE_FRAME_SIZE*num_frames;

        // Iterate over the list and find a free block
        while let Some(mut node) = current_block.take() {

            // Check if free block has to be inserted between this node and the next node (if it exists)
            if !is_between_node_and_successor(&mut node, &addr_start, &addr_end) {
                // Look at next
                let next_ptr: *mut Option<&'static mut PfListNode> = &mut node.next;
                *current_block = Some(node);

                unsafe {
                    current_block = &mut *next_ptr;
                }
                continue;
            }

            // free_block is adjacent to this and next node
            if is_adjacent_back(&mut node, &addr_start) && is_adjacent_next_front(&mut node, &addr_end) {
                let new_note_addr: *mut PfListNode = addr_start.as_mut_ptr() as *mut PfListNode;
                unsafe {
                    (*new_note_addr).size = size;
                    (*new_note_addr).next = node.next.as_mut().unwrap().next.take(); // This is okay because "is_adjacent_next_front" checks if there is a next node
                    // next node is dropped here
                }
                *current_block = Some(unsafe{ &mut *new_note_addr }); // old current_block node is dropped here
                return; 

            // free_block is adjacent to this node
            } else if is_adjacent_back(&mut node, &addr_start) {
                let new_note_addr: *mut PfListNode = addr_start.as_mut_ptr() as *mut PfListNode;
                unsafe {
                    (*new_note_addr).size = size;
                    (*new_note_addr).next = node.next.take(); // This is okay because "is_adjacent_next_front" checks if there is a next node
                }
                *current_block = Some(unsafe{ &mut *new_note_addr }); // old current_block node is dropped here
                return; 

            // free_block is adjacent to next node
            } else if is_adjacent_next_front(&mut node, &addr_end) {
                let new_note_addr: *mut PfListNode = addr_start.as_mut_ptr() as *mut PfListNode;
                unsafe {
                    (*new_note_addr).size = size;
                    (*new_note_addr).next = node.next.as_mut().unwrap().next.take(); // This is okay because "is_adjacent_next_front" checks if there is a next node
                    // next node is dropped here
                }
                node.next = Some(unsafe{ &mut *new_note_addr });
                *current_block = Some(&mut *node);
                return; 

            // free_block is adjacent to no node
            } else {
                let new_note_addr: *mut PfListNode = addr_start.as_mut_ptr() as *mut PfListNode;
                unsafe {
                    (*new_note_addr).size = size;
                    (*new_note_addr).next = node.next.take(); // This is okay because "is_adjacent_next_front" checks if there is a next node
                }
                node.next = Some(unsafe{ &mut *new_note_addr });
                *current_block = Some(&mut *node);
                return; 
            }
        }
    }

    /// Print the list of free physical memory.
    pub fn dump_free_list(&self) {
        let mut current_block = & self.head.next;

        
        println!("Dumping free memory list:");
        // Iterate over the list
        while let Some(node) = current_block { // Take the current node
            println!("   Block start:  0x{:x}, block end:  0x{:x}, block size: {}", node.start_addr().raw(), node.end_addr().raw(), node.size);
            // let next_ptr: *mut Option<&'static mut PfListNode> = &mut node.next; // Get next
            current_block = &node.next; // Put the node back into the list

            // unsafe {
            //     current_block = &mut *next_ptr; // Set current to next
            // }
        }
        
        println!("");

    }
}

/// Returns true if the addr_start and addr_end are between the node and its successor
fn is_between_node_and_successor(node: &mut &'static mut PfListNode, addr_start: &PhysAddr, addr_end: &PhysAddr) -> bool {
    if node.next.is_some() {
        return addr_start >= &node.end_addr() && addr_end <= &node.next.as_mut().unwrap().start_addr();
    }
    return addr_start >= &node.end_addr();
}

fn fill_block_with_zeros(node: &'static mut PfListNode) {
    todo!()
    /*
     * Hier muss Code eingefuegt werden
     */
}

fn is_adjacent_next_front(node: &mut &'static mut PfListNode, addr_end: &PhysAddr) -> bool {
    if let Some(next_node) = node.next.as_mut()
    {
        return next_node.start_addr().raw() == addr_end.raw(); //TODO: Check if this works
    }
    return false;
}

fn is_adjacent_back(node: &mut &'static mut PfListNode, addr_start: &PhysAddr) -> bool {
    return node.end_addr().raw() == addr_start.raw(); //TODO: Check if this works
}
