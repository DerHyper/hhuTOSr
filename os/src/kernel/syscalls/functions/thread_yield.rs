use crate::kernel::threads::scheduler::get_scheduler;

pub extern "C" fn sys_thread_yield() {
    get_scheduler().yield_cpu();
}
