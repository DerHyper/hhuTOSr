#![no_std]

use core::panic::PanicInfo;
use usrlib::user_api::usr_process_get_id;
use usrlib::user_api::usr_print;

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_print("Hello from Process with ID: ");

    let pid: usize = usr_process_get_id();
    let mut buf = itoa::Buffer::new();
    let pid_str = buf.format(pid);
    usr_print(pid_str);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
