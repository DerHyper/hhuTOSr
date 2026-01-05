use crate::kernel::threads::{scheduler::get_scheduler, thread::Thread};

/// Starts a new user thread
pub fn run() {
    let user_thread = Thread::new_user_thread(user_fallback_fn);
    let scheduler = get_scheduler();
    scheduler.ready(user_thread);
    scheduler.schedule();
}

fn user_fallback_fn() {
    print!("user_fallback_fn called");
}