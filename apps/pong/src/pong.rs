#![no_std]
mod ball;
mod frame;
mod player;
mod pong_game;
mod sound_fx;

use core::panic::PanicInfo;

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    pong_game::run();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
