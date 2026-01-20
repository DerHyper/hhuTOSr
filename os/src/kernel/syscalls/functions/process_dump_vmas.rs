use crate::kernel::{processes::process::{self, Process}, threads::scheduler::get_scheduler};

pub extern "C" fn sys_dump_vmas() {
    let process_id = get_scheduler().get_active_pid();
    process::dump_vmas(process_id);
}
