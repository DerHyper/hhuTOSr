use core::ptr;
use x86_64::structures::paging::{FrameAllocator, PageSize};

use crate::consts::{PAGE_SIZE, STACK_SIZE, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
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
        /*
         * Hier muss Code eingefuegt werden
         */
        return PageFlags::DIRTY;
    }

    fn user_flags() -> Self {
        /*
         * Hier muss Code eingefuegt werden
         */
        return PageFlags::DIRTY;
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
    entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    /// Set up a mapping from `virt_addr` to `num_pages` pages at the given `level`.
    /// If `kernel` is true, the pages will be mapped 1:1 to their physical addresses
    /// (virt_addr == phys_addr). Otherwise, new physical frames will be allocated
    /// for the mapping, using the frame allocator.
    /// returns how man pages where allocated
    fn map(&mut self, virt_addr: u64, num_pages: usize, kernel: bool) -> usize {
        let mut num_mapped_pages = 0;
        for i in 0..num_pages {
            let current_virt_addr = virt_addr + (i*PAGE_SIZE) as u64;

            let paging_l4_pml4e = current_virt_addr >> 39 & 0x1FF; // 9 bit paging-level index: Page map level 4
            let paging_l3_pdpte = current_virt_addr >> 30 & 0x1FF;
            let paging_l2_pde = current_virt_addr >> 21 & 0x1FF;
            let paging_l1_pte = current_virt_addr >> 12 & 0x1FF;
            let offset = current_virt_addr & 0xFFF; // 12 bit

            // Get to level 1
            let l4_entry = &mut self.entries[paging_l4_pml4e as usize];
            let l3_page_table = get_or_create_next_level_page_table(l4_entry);

            let l3_entry = &mut l3_page_table.entries[paging_l3_pdpte as usize];
            let l2_page_table = get_or_create_next_level_page_table(l3_entry);

            let l2_entry = &mut l2_page_table.entries[paging_l2_pde as usize];
            let l1_page_table = get_or_create_next_level_page_table(l2_entry);

            let mut l1_entry = &mut l1_page_table.entries[paging_l1_pte as usize];

            if kernel {
                // 1:1 mapping
                l1_entry.set_addr(PhysAddr::new(current_virt_addr));
                update_frame_flags(&mut l1_entry);
                num_mapped_pages += 1;
                continue;
            } 

            // Alloc new physical frames
            let frame = unsafe { FRAME_ALLOCATOR.lock().alloc_block(num_pages)};
            if let Some(frame_addr) = frame
            {
                l1_entry.set_addr(frame_addr-offset);

                update_frame_flags(&mut l1_entry);
                num_mapped_pages += 1;
                continue;
            }

            // Failed to alloc frame
            continue;
        }

        return num_mapped_pages;
    }
}

fn get_or_create_next_level_page_table(entry: &mut PageTableEntry) -> &mut PageTable {
    // Alloc page frames in not already present
    let flags = entry.get_flags();
    if !flags.contains(PageFlags::PRESENT) {
        let frame_addr = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).unwrap() };
        entry.set_addr(frame_addr);
    }

    update_frame_flags(entry);
    
    // get next page Table
    let next_page_table = unsafe {
        entry.get_addr().as_mut_ptr::<PageTable>().as_mut().unwrap()
    };
    next_page_table
}

/// Set frags according to specs
fn update_frame_flags(entry: &mut PageTableEntry) { 
    let mut flags = entry.get_flags();
    flags.set(PageFlags::PRESENT | PageFlags::WRITEABLE, true);
    entry.set_flags(flags);
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

        pml4.map(0, num_pages, true);
        pml4
    }
}

pub unsafe fn map_user_stack(pml4_table: &mut PageTable) -> *mut u8 {
    /*
     * Hier muss Code eingefuegt werden
     */

    return core::ptr::null_mut();
}