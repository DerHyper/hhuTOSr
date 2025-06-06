use crate::devices::cga;
use crate::kernel::threads::scheduler;
use crate::kernel::threads::thread::{self, Thread};
use crate::kernel::threads;

static mut TARGET_ID_1 :usize = 0;
static mut TARGET_ID_2 :usize = 0;

fn thread_entry() {
    let mut i = 0;
    let sched = scheduler::get_scheduler();

    // Loop indefinitely, printing the thread ID and a counter
    // position on the screen is determined by the thread ID
    loop {
        print_thread(i);
        i += 1;
    }
}

fn thread_entry_killer() {
    let mut i = 0;
    let sched = scheduler::get_scheduler();

    // Loop 1000 times, printing the thread ID and a counter
    while i < 1000 {
        print_thread(i);
        sched.yield_cpu(); // Switch to the next thread
        i += 1;
    }

    // Kill other threads
    unsafe {
        sched.kill(TARGET_ID_1);
        sched.kill(TARGET_ID_2);
    }

    // Loop 1000 times, printing the thread ID and a counter
    while i < 2001 {
        print_thread(i);
        sched.yield_cpu(); // Switch to the next thread
        i += 1;
    }

    sched.exit(); // exit scheduler (whould happen anyway)
}

/// Prints Threads to CGA depending on thier id
fn print_thread(i: i32) {
    let print_offset = 5 + scheduler::get_scheduler().get_active_tid();
    cga::CGA.lock().setpos(5,print_offset);
    println!("Thread [{}]: {}", scheduler::get_scheduler().get_active_tid(), i);
}

pub fn run() {
    
    // Create Therads
    let thread1 = Thread::new(thread_entry);
    let thread2 = Thread::new(thread_entry);
    let thread3 = Thread::new(thread_entry_killer);

    // Set threads to be killed
    unsafe {
        TARGET_ID_1 = thread1.get_id();
        TARGET_ID_2 = thread2.get_id();
    }
    
    // Add threads to scheduler
    let sched = scheduler::get_scheduler();
    sched.ready(thread1);
    sched.ready(thread2);
    sched.ready(thread3);

    // Start scheduler
    println!("Coroutine Demo:");
    threads::scheduler::get_scheduler().schedule();
}