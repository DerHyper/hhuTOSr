#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use core::panic::PanicInfo;
use usrlib::{allocator, consts::{USER_HEAP_SIZE, USER_HEAP_VIRT_START}, println, user_api::{usr_dump_vmas, usr_map_heap}};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    // Test usr_dump_vmas
    usr_dump_vmas();

    // Test usr_map_heap
    usr_map_heap(USER_HEAP_VIRT_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_VIRT_START as usize, USER_HEAP_SIZE);

    // Test usr_dump_vmas
    usr_dump_vmas();

    // Dynamic memory allocation
    let b = Box::new(1234);
    println!("Allocated Box");
    let mut v = Vec::with_capacity(16);
    for i in 0..16 {
        v.push(i);
    }
    println!("Allocated 16 Vecs");
    println!("See console for more info");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {
        println!("Paniced at dump_vmas_app");
    }
}
