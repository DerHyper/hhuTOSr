use crate::devices::cga::{CGA_COLUMNS, CGA_ROWS};
use crate::devices::cga;

fn print_frame(frame : [[char;CGA_COLUMNS];CGA_ROWS]) {
    let mut cga_lock = cga::CGA.lock();
    for y in 0..CGA_ROWS {
        for x in 0..CGA_COLUMNS {
            cga_lock.print_byte_at_nowrapping(frame[y][x] as u8, x, y);
        }
    }
    
}

pub fn run() {
    let bar_char = 0xDB as char; // '█' in Code page 437
    let space_char = ' ';
    let ball_char = 0x09 as char; // '○' in Code page 437
    let mut frame = [[bar_char; CGA_COLUMNS];CGA_ROWS];
    print_frame(frame);
}