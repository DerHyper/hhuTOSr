#![no_std]
mod ball;
mod frame;
mod player;
mod pong_game;
mod sound_fx;
mod game_object;

use core::panic::PanicInfo;

use crate::frame::Frame;

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    let mut frame = Frame::new();
    pong_game::run();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
