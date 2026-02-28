use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_cga::Color;

use crate::geometrics::Renderable;

/// A general game object that can be drawn on the screen. 
/// It has a position, a symbol, a color and a size in each direction.
pub struct GameObject {
    pub x: f32,
    pub y: f32,
    pub symbol: char,
    pub color: Color,
    pub size_up: f32,
    pub size_down: f32,
    pub size_left: f32,
    pub size_right: f32
}

impl GameObject {
    pub const fn new(x: f32, y: f32, symbol: char, color: Color) -> GameObject {
        GameObject {
            x,
            y,
            symbol,
            color,
            size_up: 0.,
            size_down: 0.,
            size_left: 0.,
            size_right: 0.
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