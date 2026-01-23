use core::ptr;
use x86_64::structures::paging::{FrameAllocator, PageSize};

use crate::consts::{PAGE_SIZE, STACK_SIZE, USER_CODE_VIRT_START, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
use crate::kernel::{self, multiboot};
use crate::kernel::interrupts::InterruptStackFrame;
use crate::kernel::interrupts::intdispatcher::{INT_VECTORS, InterruptVector};
use crate::kernel::paging::frames::{PhysAddr, FRAME_ALLOCATOR};

const PAGE_TABLE_ENTRIES: usize = 512;

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct PageFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITEABLE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
    }
}

impl PageFlags {
    fn kernel_flags() -> Self {
        PageFlags::PRESENT
        | PageFlags::WRITEABLE
    }

    fn user_flags() -> Self {
        PageFlags::PRESENT
        | PageFlags::WRITEABLE
        | PageFlags::USER
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    fn new(addr: PhysAddr, flags: PageFlags) -> Self {
        let addr: u64 = addr.into();
        Self(addr | flags.bits())
    }

    pub fn set(&mut self, addr: PhysAddr, flags: PageFlags) {
        *self = PageTableEntry::new(addr, flags);
    }

    pub fn get_flags(&self) -> PageFlags {
        PageFlags::from_bits_truncate(self.0)
    }

    pub fn set_flags(&mut self, flags: PageFlags) {
        *self = PageTableEntry::new(self.get_addr(), flags);
    }

    pub fn get_addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000f_ffff_ffff_f000)
    }

    pub fn set_addr(&mut self, addr: PhysAddr) {
        *self = PageTableEntry::new(addr, self.get_flags());
    }
}

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[addr={:?}, flags={:?}]",
            self.get_addr(),
            self.get_flags()
        )
    }
}

#[repr(transparent)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    /// Set up a mapping from `virt_addr` to `num_pages` pages at the given `level`.
    /// If `kernel` is true, the pages will be mapped 1:1 to their physical addresses
    /// (virt_addr == phys_addr). 
    /// If `phys_addr` is None() new physical frames will be allocated for the mapping, 
    /// using the frame allocator. Otherwise the frames at `phys_addr` will be used.
    /// returns how man pages where allocated
    fn map(&mut self, virt_addr: u64, num_pages: usize, phys_addr: Option<PhysAddr>, kernel: bool) -> usize {
        let mut num_mapped_pages = 0;
        for current_page_num in 0..num_pages {

            let current_virt_addr = virt_addr + (current_page_num*PAGE_SIZE) as u64;

            if current_virt_addr == 0 {
                continue; // Skip null pointer
            }

            let pml4_index = (current_virt_addr >> 39 & 0x1FF) as usize; // 9 bit paging-level index: Page map level 4
            let pdpt_index = (current_virt_addr >> 30 & 0x1FF) as usize;
            let pd_index = (current_virt_addr >> 21 & 0x1FF) as usize;
            let pt_index = (current_virt_addr >> 12 & 0x1FF) as usize;

            // Get to level 1
            let pml4_entry = &mut self.entries[pml4_index];
            let pdpt = get_or_create_page_table(pml4_entry, &mut num_mapped_pages, kernel);

            let pdpt_entry = &mut pdpt.entries[pdpt_index];
            let pd = get_or_create_page_table(pdpt_entry, &mut num_mapped_pages, kernel);

            let pd_entry = &mut pd.entries[pd_index];
            let pt = get_or_create_page_table(pd_entry, &mut num_mapped_pages, kernel);

            let pt_entry = &mut pt.entries[pt_index];

            if kernel {
                // 1:1 mapping
                pt_entry.set(PhysAddr::new(current_virt_addr), PageFlags::kernel_flags());
                continue;
            }

            if phys_addr.is_some() {
                // Use existing physical frames
                let current_phys_addr = phys_addr.unwrap() + (current_page_num*PAGE_SIZE);
                pt.entries[pt_index as usize].set(current_phys_addr, PageFlags::user_flags());
            } else {
                // Alloc new physical frames
                let frame = unsafe { 
                    FRAME_ALLOCATOR
                    .lock()
                    .alloc_block(1)
                    .expect("Failed to allocate new physical user frames")
                };

                pt_entry.set(frame, PageFlags::user_flags());
            }
            
            
            num_mapped_pages += 1;
            continue;
        }

        return num_mapped_pages;
    }
}

/// Returns the PageTable from a PageTableEntry.
/// Allocs a page frames in not already present
fn get_or_create_page_table(entry: &mut PageTableEntry, allocated_pages: &mut usize, kernel: bool) -> &'static mut PageTable {
    // Alloc page frames in not already present
    if !entry.get_flags().contains(PageFlags::PRESENT) {
        let frame_addr = unsafe { 
            FRAME_ALLOCATOR
            .lock()
            .alloc_block(1)
            .expect("Failed to allocate page table") 
        };
        
        // Set kernel/user flag
        if kernel {
            entry.set(frame_addr, PageFlags::kernel_flags());
        } else {
            entry.set(frame_addr, PageFlags::user_flags());
        };
        
        *allocated_pages += 1;
    
    } else if !kernel {
        // Propagate USER flag if table already exists
        let new_flags = entry.get_flags() | PageFlags::USER;
        entry.set_flags(new_flags);
    }
    
    // get next page Table
    let next_page_table = unsafe {
        entry
        .get_addr()
        .as_mut_ptr::<PageTable>()
        .as_mut()
        .unwrap()
    };
    next_page_table
}


pub fn read_cr3() -> &'static mut PageTable {
    let value: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) value);
    }

    unsafe {
        PhysAddr::new(value & 0xffff_ffff_ffff_f000)
            .as_mut_ptr::<PageTable>()
            .as_mut()
            .unwrap()
    }
}

pub unsafe fn write_cr3(pml4: &PageTable) {
    let addr: u64 = ptr::from_ref(pml4) as u64;
    unsafe {
        core::arch::asm!("mov cr3, {}", in(reg) addr);
    }
}

pub fn init_kernel_tables() -> &'static mut PageTable {
    let max_phys_addr = FRAME_ALLOCATOR.lock().get_max_phys_addr();
    let num_pages = (max_phys_addr.raw() as usize + PAGE_SIZE - 1) / PAGE_SIZE;

    unsafe {
        let pml4 = FRAME_ALLOCATOR.lock()
                .alloc_block(1)
                .expect("Failed to allocate frame for PML4!")
                .as_mut_ptr::<PageTable>()
                .as_mut()
                .unwrap();

        pml4.map(0, num_pages, None, true);
        pml4
    }
}

/// Sets up a mapping for the user stack.
/// Returns the stacks virtual address
pub unsafe fn map_user_stack(pml4_table: &mut PageTable) -> *mut u8 {
    // Rounded up, because int-division may yield 1 page to few
    // eg. "6KB Stack / 4KB Pages = 1 Page" but 2 are needed
    let num_pages = (STACK_SIZE + PAGE_SIZE - 1) / PAGE_SIZE; 

    // Map user stack pages
    pml4_table.map(USER_STACK_VIRT_START as u64, num_pages, None, false);

    return USER_STACK_VIRT_START as *mut u8;
}

/// Sets up a mapping for a user app.
/// Returns the apps virtual address
pub unsafe fn map_user_app(pml4_table: &mut PageTable, num_pages: usize, phys_addr: PhysAddr) -> *mut u8 {

    // Map user app
    pml4_table.map(USER_CODE_VIRT_START as u64, num_pages, Some(phys_addr), false);
    return USER_CODE_VIRT_START as *mut u8;
}

/// Sets up a mapping for a user heap.
pub unsafe fn map_user_heap(pml4_table: &mut PageTable, user_heap_start: u64, user_heap_size: usize) {

    let num_pages =  (user_heap_size + PAGE_SIZE - 1) / PAGE_SIZE;

    pml4_table.map(user_heap_start, num_pages, None, false);
}

/// Aligns a address to the corresponding page address
/// Source https://stackoverflow.com/questions/3023909/what-is-the-trick-in-paddress-page-size-1-to-get-the-pages-base-address
pub fn aligne_to_page(addr: u64) -> u64 {
    addr & !(PAGE_SIZE as u64 - 1)
}

/// Checks if the `fault_addr` of a Page Fault is within the user stack
/// If so, the matching page is mapped to the page table
/// returns if `fault_addr` of a Page Fault is within the user stack
fn check_and_grow_user_stack(fault_addr: u64) -> bool {

    if fault_addr < USER_STACK_VIRT_START as u64 || fault_addr >= USER_STACK_VIRT_END as u64 {
        // Is not within user stack
        kprintln!("User stack limit was reached");
        return false;
    }

    // Map new page
    let current_page_table = read_cr3();
    let page_addr = aligne_to_page(fault_addr);
    current_page_table.map(page_addr, 1, None, false);
    kprintln!("User stack size was increased");

    return true;
}

/// This function is called from the IDT syscall handler (interrupt 0x0E).
/// Throws a panic containing the address of the instruction that 
/// caused the page fault, which is written in the c2 register
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2);
    }

    panic!(
        "Page fault
        faulting address: {:#x}
        instruction pointer: {:#x}
        error code: {:#x}",
        cr2,
        stack_frame.instruction_pointer as u64,
        error_code
    );
}