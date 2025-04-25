use alloc::boxed::Box;

use crate::kernel::allocator;
use crate::keyboard;
use crate::cga;


pub fn run () {
    kprintln!("starting heap demo");
    let n_texts: usize = 2;

    demo_alloc_box(1,n_texts);
    ask_for_input();
    demo_oversized_alloc_box(2,n_texts);
}

fn ask_for_input() {
    println!("Press a key to continue");

    // Wait for key press
    let mut keyboard = keyboard::KEYBOARD.lock();
    let invalid_key = Default::default();
    let mut key = keyboard.key_hit();
    while key == invalid_key{
        key = keyboard.key_hit();
    }

    let mut cga = cga::CGA.lock();
    cga.clear();
}

fn demo_alloc_box(cur_n: usize, max_n: usize) {
    println!("Heap Demo {}/{}: Allocate 2 structs unsing Box::new", cur_n, max_n);
    println!("=================================================\n");

    allocator::dump_free_list();

    struct Test {
        a: u64,
        b: u64,
    }
    let test1 = Test { a: 1, b: 2 };
    let test2 = Test { a: 3, b: 4 };

    let heap: Box<[Test;2]> = Box::new([test1, test2]);
    println!("Added structs:\n   test1: a={}, b={}\n   test2: a={}, b={}\n", heap[0].a, heap[0].b, heap[1].a, heap[1].b);

    allocator::dump_free_list();
}

fn demo_oversized_alloc_box(cur_n: usize, max_n: usize) {
    println!("Heap Demo {}/{}: Allocate 2 structs unsing Box::new",cur_n, max_n);
    println!("=================================================\n");

    allocator::dump_free_list();

    // Save Structs in Heap
    #[repr(C)]
    struct Dummy64([u8; 64]);
    let a = Box::new(Dummy64([0u8; 64]));
    let b = Box::new(Dummy64([0u8; 64]));
    let c = Box::new(Dummy64([0u8; 64]));



    println!("Added Dummy structs:\n   [a] [Deleted b] [c]\n");

    allocator::dump_free_list();
}
