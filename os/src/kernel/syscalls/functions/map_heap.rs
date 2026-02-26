use usrlib::allocator;

use crate::{consts::PAGE_SIZE, kernel::{paging::{frames::PhysAddr, pages::{PageFlags, PageTable, PageTableEntry, map_user_heap}}, processes::{self, process::Process, vma::{self, VMA}}, threads::scheduler::get_scheduler}};
use crate::kernel::threads::scheduler;
use crate::allocator::ALLOCATOR;

pub extern "C" fn sys_map_heap(user_heap_start: u64, user_heap_size: usize) {
    // Map user heap
    let thread = scheduler::get_current_thread();
    let pml4_table = &mut thread.page_table;
    unsafe { map_user_heap(pml4_table, user_heap_start, user_heap_size) };
    
    // Create VMA for Heap
    let pid = thread.get_process_id();
    let num_pages_to_heap_end = (user_heap_start as usize + user_heap_size + PAGE_SIZE - 1) / PAGE_SIZE;
    let user_heap_end = (num_pages_to_heap_end * PAGE_SIZE) as u64;
    let vma = VMA::new(user_heap_start, user_heap_end, vma::VmaType::Heap);
    processes::process::add_vma(pid, vma);

    // Init user allocator with heap
    unsafe { ALLOCATOR.lock().init(user_heap_start as usize, user_heap_end as usize) };
}
