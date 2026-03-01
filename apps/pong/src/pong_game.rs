use core::arch::asm;

use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_api::{self, usr_get_char, usr_get_system_time, usr_try_get_char};
use usrlib::user_cga::{self, Color};
use crate::game_object::GameObject;
use crate::geometrics::{Point, Renderable};
//use crate::devices::{cga, pit};
use crate::player::{self, Player};
use crate::frame::{self, Frame};
use crate::ball::{self, Ball, BallEvent};
//use crate::library::input;
use crate::sound_fx;
use crate::upgrades::upgrade::UpgradeType;
use crate::upgrades::upgrade_manager::{self, UpgradeManager};

const MIN_WINNING_POINTS: u16 = 11;
const BAR_LENGTH: f32 = 4.;
const BAR_THICKNESS: f32 = 0.9;

const LEFT_SIDE: u16 = 0;
const RIGHT_SIDE: u16 = (CGA_COLUMNS as u16) - 1;
const Y_MIDDLE: u16 = (CGA_ROWS/2) as u16;
const X_MIDDLE: u16 = (CGA_COLUMNS/2) as u16;
const STD_BALL_SPEED_X: f32 = 0.5;
const STD_BALL_SPEED_Y: f32 = 0.25;
const STD_BALL_SPEED: f32 = 0.7;
const RESET_TIME_AFTER_GOAL: usize = 700;
const MENU_COLOR: Color = Color::LightGreen;

const MS_BETWEEN_FRAMES: usize = 33; // 30 FPS

/// Starts the game
pub fn run() {
    loop {
        let mut game_iteration = GameIteration::new();
        game_iteration.run_game_interation();
    }
}

pub struct GameIteration {
    frame: Frame,
    player_1: Player,
    player_2: Player,
    balls: [Option<Ball>; 3],
    upgrade_manager: UpgradeManager
}

impl GameIteration {
    pub fn new() -> GameIteration {
        GameIteration {
            frame: Frame::new(),
            player_1: Player::new(LEFT_SIDE as f32+1., Y_MIDDLE as f32, BAR_LENGTH , BAR_THICKNESS),
            player_2: Player::new(RIGHT_SIDE as f32-1., Y_MIDDLE as f32, BAR_LENGTH, BAR_THICKNESS),
            balls: [Some(Ball::new((CGA_COLUMNS/2) as f32, Ball::get_random_start_y(), 1 as u8)), None, None],
            upgrade_manager: UpgradeManager::new()
        }
    }

    /// Starts a new itteration of the game. 
    pub fn run_game_interation(&mut self) {
        // Init Game Objects
        if let Some(ball) = self.balls[0].as_mut() {
            ball.set_movement_direction(Point::new(STD_BALL_SPEED_X, STD_BALL_SPEED_Y));
            ball.set_speed(STD_BALL_SPEED);
        }

        // Show Start Screen
        self.show_start_screen();

        // Hide cursor
        //TODO: cga::CGA.lock().setpos(CGA_COLUMNS, CGA_ROWS);
            
        // wait for start input
        while !usr_get_char().eq_ignore_ascii_case(&'W') { // Bussy-Polling
            unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
        };
            
        // Game loop
        let mut last_frame_time =  usr_get_system_time();
        while !self.is_game_end() {
            if !self.check_next_frame_time(&mut last_frame_time) {
                unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
                continue; // Bussy-Polling
            }

            self.run_pipeline();
        
            self.draw_frame();
        }
            
        // Show End Screen
        self.show_end_screen();

        // wait for restart input
        while !usr_get_char().eq_ignore_ascii_case(&'R') { // Bussy-Polling
            unsafe{ asm!("pause"); } // TODO: Check if this makes a difference
        };
    }

    fn show_end_screen(&mut self) {
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
        if self.player_1.points > self.player_2.points {
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
    fn is_game_end(&mut self) -> bool {
        return self.player_1.points >= MIN_WINNING_POINTS || self.player_2.points >= MIN_WINNING_POINTS;
    }

    /// Write PONG at the screen together with instructions
    fn show_start_screen(&mut self) {
        // Clear Screen
        user_cga::clear_screen();

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
    fn check_next_frame_time(&mut self, last_frame_time: &mut usize) -> bool {
        let current_time = usr_get_system_time();
        if current_time - *last_frame_time <= MS_BETWEEN_FRAMES {
            return false;
        }
        *last_frame_time = current_time;
        true
    }

    /// Runs the pyhsics and event pipeline 
    fn run_pipeline(&mut self) {
        self.run_player_input();
        self.move_ball();
        self.check_ball_hit_goal();
        self.upgrade_manager.update();
    }

    /// Checks if goal was hit, if so, update player score
    fn check_ball_hit_goal(&mut self) {
        for mut ball in self.balls.iter_mut().flatten() {
            // Player 2 scored goal
            if ball.object.x < (LEFT_SIDE as f32) -0.1 {
                ball_hit_goal(&mut ball, &mut self.player_2);
                return;

            // Player 1 scored goal
            } else if ball.object.x > (RIGHT_SIDE as f32) +0.1 {
                ball_hit_goal(&mut ball,&mut self.player_1);
            }
        }
    }

    /// Move ball by one step
    fn move_ball(&mut self) {
        let mut do_spawn_ball = false;

        for mut ball in self.balls.iter_mut().flatten() {
            let event = ball.move_step(&mut self.player_1, &mut self.player_2, &mut self.upgrade_manager);
            match event {
                BallEvent::SpawnBall => do_spawn_ball = true,
                _ => {}
            }
        }

        if do_spawn_ball {
            self.spawn_ball()
        }
    }

    /// Spawns a now ball if there is space
    pub fn spawn_ball(&mut self)
    {
        for mut ball in self.balls.iter_mut() {
            if ball.is_none() {
                ball = &mut Some(Ball::new((CGA_COLUMNS/2) as f32, Ball::get_random_start_y(), 1 as u8));
                return;
            }
        }
    }

    /// Poll player input, chance input accordingly
    fn run_player_input(&mut self) {
        let last_key = usr_try_get_char();
        if let key = last_key {
            match key.to_ascii_uppercase() {
                'W' => self.player_1.up(),
                'S' => self.player_1.down(),
                'I' => self.player_2.up(),
                'K' => self.player_2.down(),
                _=>()
            }
        }
    }

    /// Calculates and prints a new frame that shows the current game state
    fn draw_frame(&mut self) {
        self.frame = Frame::new();
        self.frame.draw_middle_line();
        if self.upgrade_manager.instantiated_upgrade.upgrade_type != UpgradeType::None {
            self.frame.draw_renderable(&self.upgrade_manager.instantiated_upgrade.object, self.upgrade_manager.get_symbol(), self.upgrade_manager.get_color());
        }
        self.frame.draw_renderable(&self.player_1.object, self.player_1.symbol, self.player_1.color);
        self.frame.draw_renderable(&self.player_2.object, self.player_2.symbol, self.player_2.color);
        self.frame.draw_score(&self.player_1, &self.player_2);
        for mut ball in self.balls.iter_mut().flatten() {
            self.frame.draw_renderable(&ball.object, ball.symbol, ball.color);
        }
        self.frame.print_frame();
    }
}

/// Call if ball hit a goal, resets ball, scores player score and plays sound effect
pub fn ball_hit_goal(ball: &mut Ball,player: &mut Player) {
    ball.set_position((CGA_COLUMNS/2) as f32, Ball::get_random_start_y());
    ball.set_movement_direction(Point::new(STD_BALL_SPEED_X, STD_BALL_SPEED_Y));
    ball.set_speed(STD_BALL_SPEED);
    ball.randomize_movement_direction();
    player.score_point();
    sound_fx::play_score_point();
    // TODO: pit::wait(RESET_TIME_AFTER_GOAL);
}