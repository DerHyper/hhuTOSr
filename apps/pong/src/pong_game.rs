use core::arch::asm;

use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_api::{self, usr_get_char, usr_get_system_time};
use usrlib::user_cga::{self, Color};
//use crate::devices::{cga, pit};
use crate::player::{self, Player};
use crate::frame::{self, Frame};
use crate::ball::{self, Ball};
//use crate::library::input;
use crate::sound_fx;

const MIN_WINNING_POINTS: u16 = 11;
const BAR_LENGTH: u16 = 5;
const BAR_THICKNESS: f32 = 1.;

const LEFT_SIDE: u16 = 0;
const RIGHT_SIDE: u16 = (CGA_COLUMNS as u16) - 1;
const Y_MIDDLE: u16 = (CGA_ROWS/2) as u16;
const X_MIDDLE: u16 = (CGA_COLUMNS/2) as u16;
const STD_BALL_SPEED_X: f32 = 0.5;
const STD_BALL_SPEED_Y: f32 = 0.25;
const RESET_TIME_AFTER_GOAL: usize = 700;
const MENU_COLOR: Color = Color::LightGreen;

const MS_BETWEEN_FRAMES: usize = 33; // 30 FPS

/// Starts the game
pub fn run() {
    loop {
        run_game_interation();
    }
}

/// Starts a new itteration of the game. 
fn run_game_interation() {
    // Init Game Objects
    let mut frame = Frame::new();
    let mut player_1 = Player::new(LEFT_SIDE+1, Y_MIDDLE, BAR_LENGTH, BAR_THICKNESS);
    let mut player_2 = Player::new(RIGHT_SIDE-1, Y_MIDDLE, BAR_LENGTH, BAR_THICKNESS);
    let mut ball = Ball::new((CGA_COLUMNS/2) as f32, Ball::get_random_start_y());
    ball.set_movement(STD_BALL_SPEED_X, STD_BALL_SPEED_Y);

    // Show Start Screen
    show_start_screen();

    // Hide cursor
    //TODO: cga::CGA.lock().setpos(CGA_COLUMNS, CGA_ROWS);
        
    // wait for start input
    while usr_get_char().eq_ignore_ascii_case(&'W') { // Bussy-Polling
        unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
    };
        
    // Game loop
    let mut last_frame_time =  usr_get_system_time();
    while !is_game_end(&player_1, &player_2) {
        if !check_next_frame_time(&mut last_frame_time) {
            unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
            continue; // Bussy-Polling
        }

        run_pipeline(&mut player_1, &mut player_2, &mut ball);
    
        draw_frame(&mut frame, &player_1, &player_2, &mut ball);
    }
        
    // Show End Screen
    show_end_screen(&mut player_1, &mut player_2);

    // wait for restart input
    while usr_get_char().eq_ignore_ascii_case(&'R') { // Bussy-Polling
        unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
    };
}

fn show_end_screen(player_1: &Player, player_2: &Player) {
    // Write winner
    let player_1_str = [                                                
        "_____ _                    ___      _ _ _ _         ",
        "|  _  | |___ _ _ ___ ___   |_  |    | | | |_|___ ___ ",
        "|   __| | .'| | | -_|  _|   _| |_   | | | | |   |_ -|",
        "|__|  |_|__,|_  |___|_|    |_____|  |_____|_|_|_|___|",
        "            |___|                                    "
    ]; // rectangles.flf by David Villegas <mnementh@netcom.com> 12/94

    let player_2_str = [                                                
        "_____ _                    ___    _ _ _ _         ",
        "|  _  | |___ _ _ ___ ___   |_  |  | | | |_|___ ___ ",
        "|   __| | .'| | | -_|  _|  |  _|  | | | | |   |_ -|",
        "|__|  |_|__,|_  |___|_|    |___|  |_____|_|_|_|___|",
        "            |___|                                  "
    ]; // rectangles.flf by David Villegas <mnementh@netcom.com> 12/94

    let player_offset_y = 6;
    if player_1.points > player_2.points {
        user_cga::print_centered_block(&player_1_str, player_offset_y, MENU_COLOR);
    } else {
        user_cga::print_centered_block(&player_2_str, player_offset_y, MENU_COLOR);
    }

    // Call to action Restart
    let cta_str = ["Press 'R' to restart!"];
    let cta_offset_y = player_offset_y + player_1_str.len() + 4;
    user_cga::print_centered_block(&cta_str, cta_offset_y, MENU_COLOR);
}

/// Returns true if one player has won
fn is_game_end(player_1: &Player, player_2: &Player) -> bool {
    return player_1.points >= MIN_WINNING_POINTS || player_2.points >= MIN_WINNING_POINTS;
}

/// Write PONG at the screen together with instructions
fn show_start_screen() {
    // Write Pong
    let pong_str = [
        " _____   ____  _   _  _____", 
        "|  __ \\ / __ \\| \\ | |/ ____|",
        "| |__) | |  | |  \\| | |  __ ",
        "|  ___/| |  | | . ` | | |_ |",
        "| |    | |__| | |\\  | |__| |",
        "|_|     \\____/|_| \\_|\\_____|"
    ]; // Big by Glenn Chappell 4/93 -- based on Standard
    let pong_offset_y = 5;
    user_cga::print_centered_block(&pong_str, pong_offset_y, MENU_COLOR);

    // Write instructions
    let instructions = [
        "Player 1: Press 'W' and 'S' to move",
        "Player 2: Press 'I' and 'K' to move",
        "",
        "Press 'W' to start!"
    ];
    let instruction_offset_y = pong_offset_y + pong_str.len() + 3;
    user_cga::print_centered_block(&instructions, instruction_offset_y, MENU_COLOR);
}

/// Returns true if enugh time has elapsed to draw a new frame
fn check_next_frame_time(last_frame_time: &mut usize) -> bool {
    let current_time = usr_get_system_time();
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
    if ball.x < (LEFT_SIDE as f32) -0.1 {
        ball_hit_goal(ball, player_2);

    // Player 1 scored goal
    } else if ball.x > (RIGHT_SIDE as f32) +0.1 {
        ball_hit_goal(ball, player_1);
    }
}

/// Call if ball hit a goal, resets ball, scores player score and plays sound effect
fn ball_hit_goal(ball: &mut Ball, player: &mut Player) {
    ball.set_position((CGA_COLUMNS/2) as f32, Ball::get_random_start_y());
    ball.set_movement(STD_BALL_SPEED_X, STD_BALL_SPEED_Y);
    ball.randomize_movement_direction();
    player.score_point();
    sound_fx::play_score_point();
    // pit::wait(RESET_TIME_AFTER_GOAL);
}

/// Move ball by one step
fn move_ball(ball: &mut Ball, mut player_1: &mut Player, mut player_2: &mut Player) {
    ball.move_step(&mut player_1, &mut player_2);
}

/// Poll player input, chance input accordingly
fn run_player_input(player_1: &mut Player, player_2: &mut Player) {
    //TODO: let last_key = input::try_getch();
    let last_key = usr_get_char(); // Bussy-Polling, TODO: Change to event based input
    if let key = last_key {
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
    frame.draw_middle_line();
    frame.draw_player(&player_1);
    frame.draw_player(&player_2);
    frame.draw_score(&player_1, &player_2);
    frame.draw_ball(&ball);
    frame.print_frame();
}