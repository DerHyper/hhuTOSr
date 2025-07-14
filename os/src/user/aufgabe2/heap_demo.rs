use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::kernel::allocator;
use crate::keyboard;
use crate::cga;
use crate::library::input;

pub fn run () {
    kprintln!("starting heap demo");
    let n_texts: usize = 4;

    demo_alloc_box(1,n_texts);
    ask_for_input();
    demo_dealloc_box(2,n_texts);
    ask_for_input();
    demo_alloc_vec(3, n_texts);
    ask_for_input();
    demo_dealloc_vec(4, n_texts);
    println!("\n*** END OF DEMO ***");

}

fn ask_for_input() {
    println!("Press a key to continue");

    // Wait for key press
    let input = input::getch();
    let mut cga = cga::CGA.lock();
    cga.clear();
}

fn demo_alloc_box(cur_n: usize, max_n: usize) {
    kprintln!("[START DEMO {}]",cur_n);
    println!("Heap Demo {}/{}: Allocate 2 Structs using Box::new", cur_n, max_n);
    println!("=================================================\n");

    allocator::init();
    allocator::dump_free_list();

    struct Test {
        a: u64,
        b: u64,
    }
    let test1 = Test { a: 1, b: 2 };
    let test2 = Test { a: 3, b: 4 };

    let heap: Box<[Test;2]> = Box::new([test1, test2]);
    println!("Added Structs:\n   test1: a={}, b={}\n   test2: a={}, b={}\n", heap[0].a, heap[0].b, heap[1].a, heap[1].b);

    allocator::dump_free_list();
}

fn demo_dealloc_box(cur_n: usize, max_n: usize) {
    kprintln!("[START DEMO {}]",cur_n);
    println!("Heap Demo {}/{}: The 2 Structs where deallocated, because they went out of scope",cur_n, max_n);
    println!("=================================================\n");

    allocator::dump_free_list();
}

fn demo_alloc_vec(cur_n: usize, max_n: usize) {
    kprintln!("[START DEMO {}]",cur_n);
    println!("Heap Demo {}/{}: Allocate a Vec for storing 3 Structs",cur_n, max_n);
    println!("=================================================\n");

    println!("Allocate Vec");
    let mut test = Vec::new();
    
    println!("Allocate 3 Structs using Box::new\n");
    struct Test {
        a: u64,
        b: u64,
    }
    test.push(Box::new(Test { a: 1, b: 1}));
    test.push(Box::new(Test { a: 2, b: 2}));
    test.push(Box::new(Test { a: 3, b: 3}));

    allocator::dump_free_list();
}

fn demo_dealloc_vec(cur_n: usize, max_n: usize) {
    kprintln!("[START DEMO {}]",cur_n);
    println!("Heap Demo {}/{}: Vec will go out of scope",cur_n, max_n);
    println!("=================================================\n");

    allocator::dump_free_list();
}
