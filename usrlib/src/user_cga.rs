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

pub fn write_char(x: usize, y: usize, c: u8, color: u8) {
    let index = (y * 80 + x) * 2;

    unsafe {
        *CGA_BUFFER.add(index) = c;
        *CGA_BUFFER.add(index + 1) = color;
    }
}