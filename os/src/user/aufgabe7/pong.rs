use crate::devices::cga::{CGA_COLUMNS, CGA_ROWS};
use crate::devices::cga;
use crate::user::aufgabe7::player::{self, PlayerBar};
use crate::user::aufgabe7::frame::{self, Frame};
use crate::library::input;


const LEFT_SIDE: u16 = 0;
const RIGHT_SIDE: u16 = (CGA_COLUMNS as u16) - 1;
const Y_MIDDLE: u16 = (CGA_ROWS/2) as u16;
const X_MIDDLE: u16 = (CGA_COLUMNS/2) as u16;
const BAR_LENGTH: u16 = 5;

pub fn run() {
    let mut frame = Frame::new();
    let mut player_1 = PlayerBar::new(LEFT_SIDE, Y_MIDDLE, BAR_LENGTH);

    loop {
        
        run_player_input(&mut player_1);
        
        update_frame(&mut frame, &player_1);
    }
    
}

fn run_player_input(player_1: &mut PlayerBar) {
    let last_key = input::try_getch();
    if let Some(key) = last_key {
        match key.to_ascii_uppercase() {
            'W' => player_1.up(),
            'S' => player_1.down(),
            _=>()
        }
    }
}

/// Calculates and prints a new frame that shows the current game state
fn update_frame(frame: &mut Frame, player_1: &PlayerBar) {
    *frame = Frame::new();
    frame.draw_player(player_1);
    frame.print_frame();
}