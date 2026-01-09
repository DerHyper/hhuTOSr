use crate::kernel::{processes, threads::{scheduler::get_scheduler, thread::Thread}};

/// Starts a new user thread
pub fn run() {
    let scheduler = get_scheduler();
    let thread_name = "hello";
    scheduler.spawn_process(thread_name);
    scheduler.schedule();
}

fn user_fallback_fn() {
    print!("user_fallback_fn called");
}