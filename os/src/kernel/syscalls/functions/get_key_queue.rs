use crate::{devices::keyboard, kernel::syscalls::functions::get_key_queue, library::input::try_getch};

pub extern "C" fn sys_get_key_queue(user_ptr: *mut char, len: usize) -> u64 {
    let mut count = 0;

    while count < len {
        match try_getch() {
            Some(k) => {
                unsafe {
                    *user_ptr.add(count) = k;
                }
                count += 1;
            }
            None => break
        }
    }

    count as u64
}
