#![no_std]

use core::panic::PanicInfo;
use usrlib::user_api::usr_dump_vmas;

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_dump_vmas();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
