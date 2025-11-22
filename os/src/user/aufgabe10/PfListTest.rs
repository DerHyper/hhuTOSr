use alloc::boxed::Box;
use x86_64::structures::paging::frame;

use crate::{devices::cga, kernel::paging::frames::{self, PhysAddr}, library::input};

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
    let mut addr_opt = None;
    unsafe {addr_opt = frame_alloc.alloc_block(n_frames);};
    
    if let Some(addr) = addr_opt {
        println!("Allocated {} frames at {}.\n", n_frames, addr.raw());
    } else {
        println!("Could not alloc {} frames.\n", n_frames);
    }

    frame_alloc.dump_free_list();
    return addr_opt;
}

fn demo_dealloc(cur_n: usize, addr: Option<PhysAddr>, n_frames: usize) {
    kprintln!("[START DEMO {}]",cur_n);
    println!("Heap Demo {}/{}: Allocate {} frames", cur_n, N_TEXTS, n_frames);
    println!("=================================================\n");
    
    if addr.is_none() {
        println!("Could not dealloc {} frames.\n", n_frames);
        return;
    }

    let mut frame_alloc = frames::FRAME_ALLOCATOR.lock();
    unsafe { frame_alloc.free_block(addr.unwrap(), n_frames) };
    println!("Deallocated {} frames at {}.\n", n_frames, addr.unwrap().raw());
    frame_alloc.dump_free_list();
}