use crate::devices::cga;
use crate::kernel::threads::scheduler;
use crate::kernel::threads::thread::{self, Thread};

fn thread_entry() {
    println!("Thread [{}]", scheduler::get_scheduler().get_active_tid())
}

pub fn run() {
    let sched = scheduler::get_scheduler();
    let thread1 = Thread::new(thread_entry);
    sched.ready(thread1);
}