use crate::kernel::{processes, threads::{scheduler::get_scheduler, thread::Thread}};

/// Starts a new user thread
pub fn run() {
    let scheduler = get_scheduler();
    let thread_name = "hello_pid";
    scheduler.spawn_process(thread_name);
    scheduler.schedule();
}