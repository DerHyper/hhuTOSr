use crate::{devices::keyboard, kernel::syscalls::functions::get_key_queue};

pub extern "C" fn sys_get_key_queue(user_ptr: *mut char, len: usize) -> u64 {
    let keybuffer = keyboard::get_key_buffer();
    let keys = keybuffer.get_all_keys();


    let count = keys.len().min(len);

    for i in 0..count {
        unsafe {
            *user_ptr.add(i) = keys[i];
        }
    }

    count as u64
}
