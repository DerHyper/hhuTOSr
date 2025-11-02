use crate::kernel::threads::scheduler::get_scheduler;

pub extern "C" fn sys_thread_exit() {
    get_scheduler().exit();
}
