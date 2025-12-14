use crate::kernel::paging::pages::{init_kernel_tables, write_cr3};

/// Test kernel mapping in address room
pub fn run() {
    let table = init_kernel_tables();
    unsafe { write_cr3(table); }

    test_page_fault();
    
}

/// Tries to allocate and write at address 0x0 
/// This should cause a Page Fault that is catched
/// by the Page Fault Handler
fn test_page_fault() {
    unsafe {
        core::ptr::read_volatile(0 as *const u64);
    }
}