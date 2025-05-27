use crate::devices::cga;
use crate::kernel::threads::scheduler::{get_scheduler, Scheduler};
use crate::kernel::threads::thread::{self, Thread};

fn thread_entry() {

    println!("Thread [{}]", Scheduler::get_active_tid())

}

pub fn run() {

    /* Hier muss Code eingefuegt werden */
    let scheduler = Scheduler::get_scheduler();
    let thread1 = Thread::new(thread_entry);
    scheduler.ready(thread);

}