use core::char;

//use crate::{devices::cga::{self, CGA_COLUMNS, CGA_ROWS}, user::aufgabe7::{ball::{self, Ball}, player::{self, Player}}};
use crate::ball::{self, Ball};
use crate::player::{self, Player};
use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_cga;
use usrlib::user_cga::Color;

const BAR_SYMBOL: char = 0xDB as char; // '█' in Code page 437
const BAR_COLOR: Color = Color::White;
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

    /// Draw the bar of a player inside the frame
    pub fn draw_player(&mut self, player: &Player) {
        for y in player.upper_bar_end()..player.lower_bar_end()+1 {
            self.frame[y as usize][player.x as usize] = BAR_SYMBOL;
            self.color[y as usize][player.x as usize] = BAR_COLOR;
        }
    }

    /// Draw the ball
    pub fn draw_ball(&mut self, ball: &Ball) {
        let y = ball::round(ball.y);
        let x = ball::round(ball.x);
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

    pub(crate) fn draw_middle_line(&mut self) {
        let x = CGA_COLUMNS/2;
        for y in 0..CGA_ROWS {
            self.frame[y][x] = DIVIDER_LINE;
        }
    }
}