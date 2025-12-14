use crate::kernel::paging::pages::{init_kernel_tables, write_cr3};

/// Test kernel mapping in address room
pub fn run() {
    let table = init_kernel_tables();
    unsafe { write_cr3(table); }
}