use crate::kernel::threads::scheduler::get_scheduler;

pub extern "C" fn sys_thread_get_id() -> u64 {
    get_scheduler().get_active_tid() as u64
}
