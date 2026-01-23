use usrlib::allocator;

use crate::{consts::PAGE_SIZE, kernel::{paging::{frames::PhysAddr, pages::{PageFlags, PageTable, PageTableEntry, map_user_heap}}, processes::{self, process::Process, vma::{self, VMA}}, threads::scheduler::get_scheduler}};
use crate::kernel::threads::scheduler;
use crate::allocator::ALLOCATOR;

pub extern "C" fn sys_map_heap(user_heap_start: u64, user_heap_size: usize) {
    // Map user heap
    
}
