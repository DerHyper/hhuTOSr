use alloc::boxed::Box;

use crate::kernel::allocator;


pub fn run () {
    kprintln!("starting heap demo");
    println!("Heap Demo 1/1: Allocate 2 structs unsing Box::new");
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
