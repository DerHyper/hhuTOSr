use crate::devices::cga;
use crate::kernel::threads::scheduler;
use crate::kernel::threads::thread::{self, Thread};
use crate::kernel::threads;

fn thread_entry() {
    println!("Thread [{}]", scheduler::get_scheduler().get_active_tid());
    scheduler::get_scheduler().yield_cpu();
}

pub fn run() {
    let sched = scheduler::get_scheduler();
    let thread1 = Thread::new(thread_entry);
    let thread2 = Thread::new(thread_entry);
    let thread3 = Thread::new(thread_entry);
    sched.ready(thread1);
    sched.ready(thread2);
    sched.ready(thread3);

    threads::scheduler::get_scheduler().schedule();
}