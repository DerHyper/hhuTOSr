use crate::{consts::{CGA_COLUMNS, CGA_ROWS}, spinlock::Spinlock};

const CGA_BUFFER: *mut u8 = 0x3000_0000_0000 as *mut u8;

/// All 16 CGA colors.
#[repr(u8)] // store each enum variant as an u8
#[derive(Clone, Copy)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Pink       = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    LightPink  = 13,
    Yellow     = 14,
    White      = 15,
}

/// Cursor struct to keep track of the current position of the cursor on the screen.
/// This is needed to implement scrolling and line breaks but does not represent the 
/// actual hardware cursor.
pub struct Cursor {
    x: usize,
    y: usize
}

impl Cursor {
    pub const fn new() -> Cursor {
        Cursor { x: 0, y: 0 }
    }

    pub fn setpos(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn getpos(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}

pub static CURSOR: Spinlock<Cursor> = Spinlock::new(Cursor::new());

pub fn write_char(x: usize, y: usize, c: u8, color: u8) {
    let index = (y * 80 + x) * 2;

    unsafe {
        *CGA_BUFFER.add(index) = c;
        *CGA_BUFFER.add(index + 1) = color;
    }
}

/// Print byte `b` at actual position cursor position `x`,`y`
pub fn print_byte(b: u8, mut x: usize, mut y: usize, color: u8) {

    // Check for new line
    if b == '\n' as u8
    {
        x = 0;
        y = y+1;
        CURSOR.lock().setpos(x, y);
        return
    }

    // Scroll up if needed
    if y >= CGA_ROWS // Scroll Up
    {
        (x, y) = (0, CGA_ROWS-1);
        CURSOR.lock().setpos(x, y);
    }
    else if x+1 > CGA_COLUMNS // Linebrake
    {
        (x, y) = (0, y+1);
        CURSOR.lock().setpos(x, y);
    }

    // Print character
    write_char(x, y, b, color);
    CURSOR.lock().setpos(x+1, y);  
}

pub fn print(s: &str, x: usize, y: usize, color: u8) {
    CURSOR.lock().setpos(x, y);
    for byte in s.bytes() {

        let (cursor_x  , cursor_y ) = CURSOR.lock().getpos();
        match byte {
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' => print_byte(byte, cursor_x, cursor_y, color),

            // not part of printable ASCII range
            _ => print_byte(0xfe, cursor_x, cursor_y, color),
        }
    }
}

pub fn print_centered_block(lines: &[&'static str], y_offset: usize, color: Color) {
    for y in 0..lines.len() {
        let x_offset = (CGA_COLUMNS-lines[y].len())/2;
        print(lines[y], x_offset, y_offset+y, color as u8);
    }
}