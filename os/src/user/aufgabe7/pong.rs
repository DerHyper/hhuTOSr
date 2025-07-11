use crate::devices::cga::{CGA_COLUMNS, CGA_ROWS};
use crate::devices::cga;
use crate::user::aufgabe7::player::{self, PlayerBar};
use crate::user::aufgabe7::frame::{self, Frame};
use crate::devices::keyboard;

const LEFT_SIDE: u16 = 0;
const RIGHT_SIDE: u16 = (CGA_COLUMNS as u16) - 1;
const Y_MIDDLE: u16 = (CGA_ROWS/2) as u16;
const X_MIDDLE: u16 = (CGA_COLUMNS/2) as u16;
const BAR_LENGTH: u16 = 5;

pub fn run() {
    loop {
        let mut frame = Frame::new();
        let player_1 = PlayerBar::new(LEFT_SIDE, Y_MIDDLE, BAR_LENGTH);
        frame.draw_player(player_1);
        frame.print_frame();
        let last_key = keyboard::get_key_buffer().get_last_key();
        if last_key.is_none() {
            continue;
        }
        
    }
    
}