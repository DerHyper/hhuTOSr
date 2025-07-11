use crate::{devices::cga::{self, CGA_COLUMNS, CGA_ROWS}, user::aufgabe7::player::PlayerBar};

const BAR: char = 0xDB as char; // '█' in Code page 437
const SPACE: char = ' ';
const BALL: char = 0x09 as char; // '○' in Code page 437

pub struct Frame {
    frame : [[char; CGA_COLUMNS]; CGA_ROWS]
}

impl Frame {
    pub const fn new() -> Frame {
        Frame {frame : [[SPACE; CGA_COLUMNS]; CGA_ROWS]}
    }

    /// print frame to CGA
    pub fn print_frame(&mut self) {
        let mut cga_lock = cga::CGA.lock();
        for y in 0..CGA_ROWS {
            for x in 0..CGA_COLUMNS {
                cga_lock.print_byte_at_nowrapping(self.frame[y][x] as u8, x, y);
            }
        }
    }

    /// Draw the bar of a player inside the frame
    pub fn draw_player(&mut self, mut player: PlayerBar) {
        for y in player.upper_bar_end()..player.lower_bar_end()+1 {
            self.frame[y as usize][player.x as usize] = BAR;
        }
    }
}