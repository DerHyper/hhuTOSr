#![no_std]
#![feature(variant_count)]
mod ball;
mod frame;
mod player;
mod pong_game;
mod sound_fx;
mod game_object;
mod utils;
mod geometrics;
mod upgrades;

use core::panic::PanicInfo;

use usrlib::{allocator, consts::{USER_HEAP_SIZE, USER_HEAP_VIRT_START}, user_api::usr_map_heap};

use crate::frame::Frame;

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
