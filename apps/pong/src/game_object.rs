use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_cga::Color;

/// A general game object that can be drawn on the screen. 
/// It has a position, a symbol, a color and a size in each direction.
pub struct GameObject {
    x: f32,
    y: f32,
    symbol: char,
    color: Color,
    size_up: f32,
    size_down: f32,
    size_left: f32,
    size_right: f32
}

impl GameObject {
    pub const fn new(x: f32, y: f32, symbol: char, color: Color) -> GameObject {
        GameObject {
            x,
            y,
            symbol,
            color,
            size_up: 1.,
            size_down: 1.,
            size_left: 1.,
            size_right: 1.
        }
    }

    pub const fn new_sized(x: f32, y: f32, symbol: char, color: Color, size_up: f32, size_down: f32, size_left: f32, size_right: f32) -> GameObject {
        GameObject {
            x,
            y,
            symbol,
            color,
            size_up,
            size_down,
            size_left,
            size_right
        }
    }
}