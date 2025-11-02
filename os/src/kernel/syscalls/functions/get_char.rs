use crate::library::input::getch;

pub extern "C" fn sys_get_char() -> u64 {
    getch() as u64
}
