use core::arch::asm;

use crate::devices::cga::{CGA_COLUMNS, CGA_ROWS};
use crate::devices::{cga, pit};
use crate::user::aufgabe7::player::{self, PlayerBar};
use crate::user::aufgabe7::frame::{self, Frame};
use crate::user::aufgabe7::ball::{self, Ball};
use crate::library::input;

const LEFT_SIDE: u16 = 0;
const RIGHT_SIDE: u16 = (CGA_COLUMNS as u16) - 1;
const Y_MIDDLE: u16 = (CGA_ROWS/2) as u16;
const X_MIDDLE: u16 = (CGA_COLUMNS/2) as u16;
const BAR_LENGTH: u16 = 5;

const MS_BETWEEN_FRAMES: usize = 33;

// pub static mut player_1: PlayerBar = PlayerBar::new(LEFT_SIDE+1, Y_MIDDLE, BAR_LENGTH);
// pub static mut player_2: PlayerBar = PlayerBar::new(RIGHT_SIDE-1, Y_MIDDLE, BAR_LENGTH);
// pub static mut ball: Ball = Ball::new((CGA_ROWS/2) as u16, (CGA_COLUMNS/2) as u16);


pub fn run() {
    // Init Game Objects
    let mut frame = Frame::new();
    let mut player_1 = PlayerBar::new(LEFT_SIDE+1, Y_MIDDLE, BAR_LENGTH);
    let mut player_2 = PlayerBar::new(RIGHT_SIDE-1, Y_MIDDLE, BAR_LENGTH);
    let mut ball = Ball::new((CGA_COLUMNS/2) as u16, (CGA_ROWS/2) as u16);
    ball.set_movement(1, 1);


    let mut last_frame_time =  pit::get_system_time();

    loop {
        if !check_next_frame_time(&mut last_frame_time) {
            unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
            continue; // Bussy-Polling
        }

        run_pipeline(&mut player_1, &mut player_2, &mut ball);
        
        draw_frame(&mut frame, &player_1, &player_2, &mut ball);
    }
    
}

/// Returns true if enugh time has elapsed to draw a new frame
fn check_next_frame_time(last_frame_time: &mut usize) -> bool {
    let current_time = pit::get_system_time();
    if current_time - *last_frame_time <= MS_BETWEEN_FRAMES {
        return false;
    }
    *last_frame_time = current_time;
    true
}

/// Runs the pyhsics and event pipeline 
fn run_pipeline(player_1: &mut PlayerBar, player_2: &mut PlayerBar, ball: &mut Ball) {
    run_player_input(player_1, player_2);
    move_ball(ball, player_1, player_2);
    check_ball_hit_goal(ball);
}


fn check_ball_hit_goal(ball: &mut Ball) {
    if ball.x == 0 || ball.x == (CGA_COLUMNS-1) as u16 {
        *ball = Ball::new((CGA_COLUMNS/2) as u16, (CGA_ROWS/2) as u16);
        ball.set_movement(1, 1);
    }
}


fn move_ball(ball: &mut Ball, mut player_1: &mut PlayerBar, mut player_2: &mut PlayerBar) {
    ball.move_step(&mut player_1, &mut player_2);
}

fn run_player_input(player_1: &mut PlayerBar, player_2: &mut PlayerBar) {
    let last_key = input::try_getch();
    if let Some(key) = last_key {
        match key.to_ascii_uppercase() {
            'W' => player_1.up(),
            'S' => player_1.down(),
            'I' => player_2.up(),
            'K' => player_2.down(),
            _=>()
        }
    }
}

/// Calculates and prints a new frame that shows the current game state
fn draw_frame(frame: &mut Frame, player_1: &PlayerBar, player_2: &PlayerBar, ball: &Ball) {
    *frame = Frame::new();
    frame.draw_player(&player_1);
    frame.draw_player(&player_2);
    frame.draw_ball(&ball);
    frame.print_frame();
}