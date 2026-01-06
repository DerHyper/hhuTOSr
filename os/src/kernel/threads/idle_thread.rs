use crate::kernel::threads::scheduler;
use crate::kernel::threads::scheduler::{get_scheduler, Scheduler};
use crate::kernel::threads::thread::Thread;

pub static IDLE_PROCESS_ID: usize = 0;

pub fn idle_thread() {
    loop {
      kprintln!("Idle...");
		  get_scheduler().yield_cpu();
    }
}
