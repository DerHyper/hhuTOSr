use crate::kernel::threads::thread::Thread;
use crate::kernel::threads::scheduler;
use crate::kernel::threads;

pub fn hello_world() {
    println!("Hello world from a thread!");
}

pub fn run() {
    let thread = Thread::new_user_thread(hello_world);
    scheduler::get_scheduler().ready(thread);
    scheduler::get_scheduler().schedule();
}