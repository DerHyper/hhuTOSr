use core::char;

//use crate::{devices::cga::{self, CGA_COLUMNS, CGA_ROWS}, user::aufgabe7::{ball::{self, Ball}, player::{self, Player}}};
use crate::ball::{self, Ball};
use crate::game_object::{self, GameObject};
use crate::geometrics::Renderable;
use crate::player::{self, Player};
use crate::utils;
use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_cga;
use usrlib::user_cga::Color;

const SPACE: char = ' ';
// const BALL: char = 0x09 as char; // '○' in Code page 437
const BALL_SYMBOL: char = 0xDB as char; // '█' in Code page 437
const BALL_COLOR: Color = Color::White;
const DIVIDER_LINE: char = '|';
const DIVIDER_LINE_COLOR: Color = Color::LightGray;
const SCORE_Y_BUFFER: usize = 1; // Distance between opper screen edge and score
const SCORE_COLOR: Color = Color::LightRed;

pub struct Frame {
    frame : [[char; CGA_COLUMNS]; CGA_ROWS],
    color : [[Color; CGA_COLUMNS]; CGA_ROWS]
}

impl Frame {
    pub const fn new() -> Frame {
        Frame {
            frame : [[SPACE; CGA_COLUMNS]; CGA_ROWS], 
            color : [[user_cga::Color::White; CGA_COLUMNS]; CGA_ROWS] 
        }
    }

    /// print frame to CGA
    pub fn print_frame(&mut self) {
        // TODO: let mut cga_lock = cga::CGA.lock();
        for y in 0..CGA_ROWS {
            for x in 0..CGA_COLUMNS {
                usrlib::user_cga::write_char(x, y, self.frame[y][x] as u8, self.color[y][x] as u8);
            }
        }
    }

    /// Draw the ball
    pub fn draw_ball(&mut self, ball: &Ball) {
        let y = utils::round(ball.y);
        let x = utils::round(ball.x);
        self.frame[y][x] = BALL_SYMBOL;
        self.color[y][x] = BALL_COLOR;
    }
    
    /// Draw the score near the top of the screen
    pub fn draw_score(&mut self, player_1: &Player, player_2: &Player) {
        // draw p1 score
        if player_1.points < 100 {
            let p1_points_tens = char::from_digit((player_1.points/10) as u32, 10).unwrap();
            let p1_points_ones = char::from_digit((player_1.points%10) as u32, 10).unwrap();
            self.frame[SCORE_Y_BUFFER][CGA_COLUMNS/2-3] = p1_points_tens;
            self.frame[SCORE_Y_BUFFER][CGA_COLUMNS/2-2] = p1_points_ones;
            self.color[SCORE_Y_BUFFER][CGA_COLUMNS/2-3] = SCORE_COLOR;
            self.color[SCORE_Y_BUFFER][CGA_COLUMNS/2-2] = SCORE_COLOR;
        }
        
        // draw p2 score
        if player_2.points < 100 {
            let p2_points_tens = char::from_digit((player_2.points/10) as u32, 10).unwrap();
            let p2_points_ones = char::from_digit((player_2.points%10) as u32, 10).unwrap();
            self.frame[SCORE_Y_BUFFER][CGA_COLUMNS/2+2] = p2_points_tens;
            self.frame[SCORE_Y_BUFFER][CGA_COLUMNS/2+3] = p2_points_ones;
            self.color[SCORE_Y_BUFFER][CGA_COLUMNS/2+3] = SCORE_COLOR;
            self.color[SCORE_Y_BUFFER][CGA_COLUMNS/2+2] = SCORE_COLOR;
        }
    }

    /// Draws a general game object on the screen.
    pub fn draw(&mut self, game_object: &GameObject) {
        let pivot = (game_object.x, game_object.y);
        let left_bound = (pivot.0 - game_object.size_left) as usize;
        let right_bound = (pivot.0 + game_object.size_right) as usize;
        let upper_bound = (pivot.1 - game_object.size_up) as usize;
        let lower_bound = (pivot.1 + game_object.size_down) as usize;

        for y in upper_bound..lower_bound+1 {
            for x in left_bound..right_bound+1 {
                if x < 0 || x >= CGA_COLUMNS as usize || y < 0 || y >= CGA_ROWS as usize {
                    continue; // Skip out of bounds
                }
                self.frame[y as usize][x as usize] = game_object.symbol;
                self.color[y as usize][x as usize] = game_object.color;
            }
        }
    }

    /// Draws a renderable object on the screen.
    pub fn draw_renderable(&mut self, renderable : &impl Renderable, symbol: char, color: Color) {
        renderable.draw(self, symbol, color);
    }

    /// Draws a point on the screen.
    pub fn draw_point(&mut self, x: usize, y: usize, symbol: char, color: Color) {
        if x < 0 || x >= CGA_COLUMNS as usize || y < 0 || y >= CGA_ROWS as usize {
            return; // Out of bounds
        }
        self.frame[y][x] = symbol;
        self.color[y][x] = color;
    }

    pub(crate) fn draw_middle_line(&mut self) {
        let x = CGA_COLUMNS/2;
        for y in 0..CGA_ROWS {
            self.frame[y][x] = DIVIDER_LINE;
        }
    }
}