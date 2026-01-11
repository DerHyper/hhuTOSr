#![no_std]

use core::panic::PanicInfo;
use usrlib::user_api::usr_process_get_id;
use usrlib::user_api::usr_print;

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    let pid: usize = usr_process_get_id();
    let mut buf = itoa::Buffer::new();
    let s = buf.format(pid);
    loop {
        usr_print(s);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
