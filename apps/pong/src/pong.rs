#![no_std]
mod ball;
mod frame;
mod player;
mod pong_game;
mod sound_fx;
mod game_object;

use core::panic::PanicInfo;
use usrlib::{allocator, consts::{USER_HEAP_SIZE, USER_HEAP_VIRT_START}, user_api::usr_map_heap};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_map_heap(USER_HEAP_VIRT_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_VIRT_START as usize, USER_HEAP_SIZE);
    
    pong_game::run();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
