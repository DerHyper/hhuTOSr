use core::arch::asm;

use crate::devices::cga::{CGA_COLUMNS, CGA_ROWS};
use crate::devices::{cga, pit};
use crate::user::aufgabe7::player::{self, Player};
use crate::user::aufgabe7::frame::{self, Frame};
use crate::user::aufgabe7::ball::{self, Ball};
use crate::library::input;
use crate::user::aufgabe7::sound_fx;

const LEFT_SIDE: u16 = 0;
const RIGHT_SIDE: u16 = (CGA_COLUMNS as u16) - 1;
const Y_MIDDLE: u16 = (CGA_ROWS/2) as u16;
const X_MIDDLE: u16 = (CGA_COLUMNS/2) as u16;
const BAR_LENGTH: u16 = 5;
const STD_BALL_SPEED_X: i16 = 1;
const STD_BALL_SPEED_Y: i16 = 1;

const MS_BETWEEN_FRAMES: usize = 33;

// pub static mut player_1: PlayerBar = PlayerBar::new(LEFT_SIDE+1, Y_MIDDLE, BAR_LENGTH);
// pub static mut player_2: PlayerBar = PlayerBar::new(RIGHT_SIDE-1, Y_MIDDLE, BAR_LENGTH);
// pub static mut ball: Ball = Ball::new((CGA_ROWS/2) as u16, (CGA_COLUMNS/2) as u16);


pub fn run() {
    // Init Game Objects
    let mut frame = Frame::new();
    let mut player_1 = Player::new(LEFT_SIDE+1, Y_MIDDLE, BAR_LENGTH);
    let mut player_2 = Player::new(RIGHT_SIDE-1, Y_MIDDLE, BAR_LENGTH);
    let mut ball = Ball::new((CGA_COLUMNS/2) as u16, (CGA_ROWS/2) as u16);
    ball.set_movement(1, 1);
    let mut last_frame_time =  pit::get_system_time();

    // Show Start Screen
    draw_frame(&mut frame, &player_1, &player_2, &mut ball);
    write_pong();
    
    while !input::getch().eq_ignore_ascii_case(&'W') { // Bussy-Polling
        unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
    };
    

    loop {
        if !check_next_frame_time(&mut last_frame_time) {
            unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
            continue; // Bussy-Polling
        }

        run_pipeline(&mut player_1, &mut player_2, &mut ball);
        
        draw_frame(&mut frame, &player_1, &player_2, &mut ball);
    }
    
}

fn write_pong() {
    // Write Pong
    let pong_str = [
        " _____   ____  _   _  _____", 
        "|  __ \\ / __ \\| \\ | |/ ____|",
        "| |__) | |  | |  \\| | |  __ ",
        "|  ___/| |  | | . ` | | |_ |",
        "| |    | |__| | |\\  | |__| |",
        "|_|     \\____/|_| \\_|\\_____|"
        ]; // Big by Glenn Chappell 4/93 -- based on Standard

    let pong_start_x = (CGA_COLUMNS-pong_str[0].len())/2;
    let pong_start_y = 5;

    for y in 0..pong_str.len() {
        cga::CGA.lock().setpos(pong_start_x, pong_start_y+y);
        println!("{}",pong_str[y]);
    }

    // Write instructions
    let instructions = [
        "Player 1: press 'W' and 'S' to move",
        "Player 2: press 'I' and 'K' to move",
        "",
        "Press 'W' to start!"
    ];
    
    let text_start_y = pong_start_y + pong_str.len() + 3;

    for y in 0..instructions.len() {
        let text_start_x = (CGA_COLUMNS-instructions[y].len())/2;
        cga::CGA.lock().setpos(text_start_x, text_start_y+y);
        println!("{}",instructions[y]);
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
fn run_pipeline(player_1: &mut Player, player_2: &mut Player, ball: &mut Ball) {
    run_player_input(player_1, player_2);
    move_ball(ball, player_1, player_2);
    check_ball_hit_goal(ball, player_1, player_2);
}

/// Checks if goal was hit, if so, update player score
fn check_ball_hit_goal(ball: &mut Ball, player_1: &mut Player, player_2: &mut Player) {
    // Player 2 scored goal
    if ball.x == 0 {
        ball.set_position((CGA_COLUMNS/2) as u16, (CGA_ROWS/2) as u16);
        ball.set_movement(STD_BALL_SPEED_X, STD_BALL_SPEED_Y);
        player_2.score_point();
        sound_fx::play_score_point();

    // Player 1 scored goal
    } else if ball.x == (CGA_COLUMNS-1) as u16 {
        ball.set_position((CGA_COLUMNS/2) as u16, (CGA_ROWS/2) as u16);
        ball.set_movement(STD_BALL_SPEED_X, STD_BALL_SPEED_Y);
        player_1.score_point();
        sound_fx::play_score_point();
    }
}


fn move_ball(ball: &mut Ball, mut player_1: &mut Player, mut player_2: &mut Player) {
    ball.move_step(&mut player_1, &mut player_2);
}

fn run_player_input(player_1: &mut Player, player_2: &mut Player) {
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
fn draw_frame(frame: &mut Frame, player_1: &Player, player_2: &Player, ball: &Ball) {
    *frame = Frame::new();
    frame.draw_player(&player_1);
    frame.draw_player(&player_2);
    frame.draw_score(&player_1, &player_2);
    frame.draw_ball(&ball);
    frame.print_frame();
}