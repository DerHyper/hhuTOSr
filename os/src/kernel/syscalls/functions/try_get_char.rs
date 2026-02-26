use crate::library::input::{getch, try_getch};

pub extern "C" fn sys_try_get_char() -> u64 {
    match try_getch() {
        Some(c) => c as u64,
        None => '\0' as u64, // Return 0 if no key was pressed
        
    }
}
