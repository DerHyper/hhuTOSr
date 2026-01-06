use crate::kernel::threads::scheduler::get_scheduler;

pub extern "C" fn sys_process_get_id() -> u64 {
    get_scheduler().get_active_pid() as u64
}
