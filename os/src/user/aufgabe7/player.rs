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
        PlayerBar {x, y, length, min_y: 0, max_y: (CGA_ROWS-1) as u16}
    }

    /// Moves the bar up by one step (-1, scine y is 0 at top of screen)
    pub fn up(&mut self)
    {
        if self.upper_bar_end() > self.min_y {
            self.y = self.y-1;
        }
    }

    /// Moves the bar down by one step (+1, scine y is 0 at top of screen)
    pub fn down(&mut self)
    {
        if self.lower_bar_end() < self.max_y {
            self.y = self.y+1;
        }
    }

    /// Returns the y position of the upper end of the bar
    pub fn upper_bar_end(&self) -> u16
    {
        self.y - (self.length/2)
    }

    /// Returns the y position of the lower end of the bar
    pub fn lower_bar_end(&self) -> u16
    {
        self.y + (self.length/2)
    }

    /// Returns true, if the other coordinates are within the bar 
    pub fn is_colliding(&self, other_x: u16, other_y: u16) -> bool {
        let is_inside_x_range = other_x == self.x; // Bar is 1 dimensional
        let is_inside_y_range = other_y <= self.lower_bar_end() && other_y >= self.upper_bar_end();
        return is_inside_x_range && is_inside_y_range;
    }
}