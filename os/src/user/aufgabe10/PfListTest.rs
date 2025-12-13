use alloc::boxed::Box;
use x86_64::structures::paging::frame;

use crate::{consts::PAGE_FRAME_SIZE, devices::cga, kernel::paging::frames::{self, PhysAddr}, library::input};

static N_TEXTS: usize = 4;

pub fn run() {
    kprintln!("starting heap demo");

    let n_frames_1 = 2;
    let addr_1 = demo_alloc(1, n_frames_1);
    ask_for_input();

    let n_frames_2 = 5;
    let addr_2 = demo_alloc(2, n_frames_2);
    ask_for_input();

    demo_dealloc(3, addr_1, n_frames_1);
    ask_for_input();

    demo_dealloc(4, addr_2, n_frames_2);
    ask_for_input();

    // Extra tests with no output
    test_reuse_same_block();
    test_fragmentation();
    test_zero_alloc();

    println!("\n*** END OF DEMO ***");
}

fn ask_for_input() {
    println!("Press a key to continue");

    // Wait for key press
    let input = input::getch();
    let mut cga = cga::CGA.lock();
    cga.clear();
}


fn demo_alloc(cur_n: usize, n_frames: usize) -> Option<PhysAddr> {
    kprintln!("[START DEMO {}]",cur_n);
    println!("Heap Demo {}/{}: Allocate {} frames", cur_n, N_TEXTS, n_frames);
    println!("=================================================\n");

    let mut frame_alloc = frames::FRAME_ALLOCATOR.lock();
    frame_alloc.dump_free_list();

    let addr_opt = unsafe { frame_alloc.alloc_block(n_frames) };

    // ASSERT: n_frames > 0
    if n_frames > 0 {
        assert_some(&addr_opt, "alloc_block returned None for n_frames > 0");
    }

    if let Some(addr) = addr_opt {
        println!("Allocated {} frames at {}.\n", n_frames, addr.raw());

        // ASSERT: frame aligned
        assert!(
            addr.raw() % PAGE_FRAME_SIZE as u64 == 0,
            "Allocated address is not page aligned"
        );
    } else {
        println!("Could not alloc {} frames.\n", n_frames);
    }

    frame_alloc.dump_free_list();
    addr_opt
}


fn demo_dealloc(cur_n: usize, addr: Option<PhysAddr>, n_frames: usize) {
    kprintln!("[START DEMO {}]",cur_n);
    println!("Heap Demo {}/{}: Allocate {} frames", cur_n, N_TEXTS, n_frames);
    println!("=================================================\n");

    assert_some(&addr, "Attempted to dealloc None");

    let addr = addr.unwrap();
    let mut frame_alloc = frames::FRAME_ALLOCATOR.lock();

    unsafe { frame_alloc.free_block(addr, n_frames) };

    println!("Deallocated {} frames at {}.\n", n_frames, addr.raw());
    frame_alloc.dump_free_list();
}

fn test_reuse_same_block() {
    println!("TEST: reuse same block");

    let mut fa = frames::FRAME_ALLOCATOR.lock();

    let a1 = unsafe { fa.alloc_block(3) };
    assert_some(&a1, "first alloc failed");

    unsafe { fa.free_block(a1.unwrap(), 3) };

    let a2 = unsafe { fa.alloc_block(3) };
    assert_some(&a2, "second alloc failed");

    assert_physaddr_eq(
        a1.unwrap(),
        a2.unwrap(),
        "allocator did not reuse freed block",
    );
}

fn test_fragmentation() {
    println!("TEST: fragmentation");

    let mut fa = frames::FRAME_ALLOCATOR.lock();

    let a = unsafe { 
        fa.alloc_block(2) 
    }.unwrap();
    let b = unsafe { fa.alloc_block(2) }.unwrap();
    let c = unsafe { fa.alloc_block(2) }.unwrap();

    unsafe {
        fa.free_block(b, 2); // hole in middle
    }

    let b2 = unsafe { fa.alloc_block(2) };
    assert_some(&b2, "allocator ignored fragmentation");

    // Cleanup
    unsafe {
        fa.free_block(b2.unwrap(), 2); // hole in middle
        fa.free_block(a, 2);
        fa.free_block(c, 2);
    }
}

fn test_zero_alloc() {
    println!("TEST: zero-size allocation");

    let mut fa = frames::FRAME_ALLOCATOR.lock();
    let addr = unsafe { fa.alloc_block(0) };

    assert_none(&addr, "alloc_block(0) should return None");
}

fn assert_some<T>(val: &Option<T>, msg: &str) {
    assert!(val.is_some(), "ASSERT FAILED: {}", msg);
}

fn assert_none<T>(val: &Option<T>, msg: &str) {
    assert!(val.is_none(), "ASSERT FAILED: {}", msg);
}

fn assert_physaddr_eq(a: PhysAddr, b: PhysAddr, msg: &str) {
    assert!(
        a.raw() == b.raw(),
        "ASSERT FAILED: {} ({} != {})",
        msg,
        a.raw(),
        b.raw()
    );
}