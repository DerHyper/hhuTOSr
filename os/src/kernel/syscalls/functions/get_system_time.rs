use crate::devices::pit::get_system_time;

pub extern "C" fn sys_get_system_time() -> u64 {
    get_system_time() as u64
}
