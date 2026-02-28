use usrlib::{consts::CGA_ROWS, user_cga::Color};

use crate::{game_object::GameObject, geometrics::{Rect, Renderable}};

const BAR_SYMBOL: char = 0xDB as char; // '█' in Code page 437
const BAR_COLOR: Color = Color::White;

/// Player, which is represented on the screen as a bar
pub struct Player {
    //pub game_object: GameObject,
    pub collider: Rect,
    pub symbol: char,
    pub color: Color,
    pub points: u16
}

static MIN_Y: f32 = 0.;
static MAX_Y: f32 = (CGA_ROWS-1) as f32;

impl Player {
    /// Creates a new player
    pub const fn new(x: f32, y: f32, length: f32, thickness: f32 ) -> Player {
        Player {
            collider: Rect::new(x as f32, y as f32, thickness, length as f32),
            symbol: BAR_SYMBOL, 
            color: BAR_COLOR,
            points: 0
        }
    }

    /// Moves the bar up by one step (-1, scine y is 0 at top of screen)
    pub fn up(&mut self)
    {
        if self.upper_bar_end() > MIN_Y {
            self.collider.pivot.y = self.collider.pivot.y-1.0;
        }
    }

    /// Moves the bar down by one step (+1, scine y is 0 at top of screen)
    pub fn down(&mut self)
    {
        if self.lower_bar_end() < MAX_Y {
            self.collider.pivot.y = self.collider.pivot.y+1.0;
        }
    }

    /// Returns the y position of the upper end of the bar
    pub fn upper_bar_end(&self) -> f32
    {
        self.collider.pivot.y - self.collider.height/2.0
    }

    /// Returns the y position of the lower end of the bar
    pub fn lower_bar_end(&self) -> f32
    {
        self.collider.pivot.y + self.collider.height/2.0
    }

    /// Returns true, if the other coordinates are within the bar 
    pub fn is_colliding(&self, other_x: f32, other_y: f32) -> bool {
        let is_inside_x_range = other_x <= self.right_bar_end() && other_x >= self.left_bar_end();
        let is_inside_y_range = other_y <= self.lower_bar_end() as f32 && other_y >= self.upper_bar_end() as f32;
        return is_inside_x_range && is_inside_y_range;
    }

    /// Adds a point to the players score
    pub fn score_point(&mut self) {
        self.points = self.points+1;
    }
    
    fn right_bar_end(&self) -> f32 {
        self.collider.pivot.x as f32 + self.collider.width/2.0
    }
    
    fn left_bar_end(&self) -> f32 {
        self.collider.pivot.x as f32 - self.collider.width/2.0
    }
}