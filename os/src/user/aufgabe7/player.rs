use crate::devices::cga::{CGA_COLUMNS, CGA_ROWS};

pub struct PlayerBar {
    pub x: u16,
    pub y: u16,
    pub length: u16,
    pub min_y: u16,
    pub max_y: u16
}

impl PlayerBar {
    pub const fn new(x: u16, y: u16, length: u16) -> PlayerBar {
        PlayerBar {x, y, length, min_y: 0, max_y: CGA_ROWS as u16}
    }

    pub fn up(&mut self)
    {
        if self.y-(self.length/2) > 1 {
            self.y = self.y-1;
        }
    }

    pub fn upper_bar_end(&mut self) -> u16
    {
        self.y - (self.length/2)
    }

    pub fn lower_bar_end(&mut self) -> u16
    {
        self.y + (self.length/2)
    }
}