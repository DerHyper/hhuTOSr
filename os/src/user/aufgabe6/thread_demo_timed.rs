use crate::devices::{cga, pit};
use crate::kernel::threads::scheduler;
use crate::kernel::threads::thread::{self, Thread};
use crate::kernel::threads;

static MAX_THREADLOOPS_UNTIL_YIELD :i32 = 10;

fn thread_entry() {
    let mut i = 0;
    let start_time = pit::get_system_time();

    // Loop 100_000 times, printing the thread ID and a counter
    while i < 100_000 {
        let running_time = pit::get_system_time() - start_time;
        i += 1;
        print_thread(i, running_time);
        check_for_yield(i);
    }

    scheduler::get_scheduler().exit(); // exit scheduler (whould happen anyway)
}

/// Prints Threads to CGA depending on thier id
fn print_thread(i: i32, time: usize) {
    let print_offset = 5 + scheduler::get_scheduler().get_active_tid();
    let mut cga_lock = cga::CGA.lock();
    cga_lock.setpos(5,print_offset);

    println_cga!(&mut cga_lock, "Thread [{}]: {:>6} <{:0>6}ms>", scheduler::get_scheduler().get_active_tid(), i, time);
}

fn check_for_yield(i: i32) {
    if i%MAX_THREADLOOPS_UNTIL_YIELD == 0 
    {
        let sched = scheduler::get_scheduler();
        sched.yield_cpu(); // other threads can get CGA access
    }
}


pub fn run() {
    
    // Create Therads
    let thread1 = Thread::new_user_thread(thread_entry);
    let thread2 = Thread::new_user_thread(thread_entry);
    let thread3 = Thread::new_user_thread(thread_entry);
    
    // Add threads to scheduler
    let sched = scheduler::get_scheduler();
    sched.ready(thread1);
    sched.ready(thread2);
    sched.ready(thread3);

    // Start scheduler
    println!("Coroutine Demo:");
    threads::scheduler::get_scheduler().schedule();
}