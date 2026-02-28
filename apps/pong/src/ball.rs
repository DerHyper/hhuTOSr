//use crate::devices::{cga::{CGA_COLUMNS, CGA_ROWS}, pit};
use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_api::usr_get_system_time;
use usrlib::user_cga::Color;
use crate::geometrics::Point;
use crate::player::{self, Player};
use crate::sound_fx;
use crate::utils::random_range;

const BALL_SPEEDUP_MULTIPLICATOR: f32 = 1.2;
// const BALL: char = 0x09 as char; // '○' in Code page 437
const BALL_SYMBOL: char = 0xDB as char; // '█' in Code page 437
const BALL_COLOR: Color = Color::White;

/// 1 Letter big ball that moves over the screen
pub struct Ball {
    pub object: Point,
    //pub size: u16,
    pub symbol: char,
    pub color: Color,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub movement_x: f32,
    pub movement_y: f32
}

impl Ball {
    /// Creates a new ball
    pub const fn new(x: f32, y: f32) -> Ball {
        Ball {
            object: Point::new(x, y),
            symbol: BALL_SYMBOL,
            color: BALL_COLOR,
            min_x: 0.0,
            max_x: (CGA_COLUMNS -1 ) as f32,
            min_y: 0.0, 
            max_y: (CGA_ROWS-1) as f32,
            movement_x: 0.,
            movement_y: 0.
        }
    }

    /// Move the ball by one step, flipping the direction if colliding with other object.
    /// Movement direction is definded in `movement_x` and `movement_y`.
    pub fn move_step(&mut self, mut player_1: &mut Player, mut player_2: &mut Player)
    {
        self.check_collision(&mut player_1, &mut player_2);
        self.object.y = self.object.y + self.movement_y;
        self.object.x = self.object.x + self.movement_x;
    }

    /// Sets `movement_x` and `movement_y`. Move will be fulfilled after calling `move_step()`.
    pub fn set_movement(&mut self, new_x: f32, new_y: f32)
    {
        self.movement_x = new_x;
        self.movement_y = new_y;
    }

    /// Sets `x` and `y`.
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.object.y = y;
        self.object.x = x;
    }

    /// Checks if ball would clip inside a border/object in the next movement step.
    /// If that would happen, flip the movement.
    fn check_collision(&mut self, player_1: &mut Player, player_2: &mut Player) {
        // Check collision with border
        let next_y = self.object.y + self.movement_y;
        if next_y > self.max_y || next_y < self.min_y {
            self.flip_y();
            sound_fx::play_collision_border();
        }

        // Check collition with bar
        let next_x = (self.object.x + self.movement_x) as f32;
        let collides_with_player =
            player_1.is_colliding(next_x, next_y) ||
            player_2.is_colliding(next_x, next_y);
        if collides_with_player {
            self.flip_x();
            self.increase_speed();
            sound_fx::play_collision_player();
        }
    }
    
    /// Invert `movement_y`
    fn flip_y(&mut self) {
        self.movement_y = -self.movement_y;
    }

    /// Invert `movement_x`
    fn flip_x(&mut self) {
        self.movement_x = -self.movement_x;
    }

    /// Returns a random number within the y range
    pub fn get_random_start_y() -> f32 {
        let rand_within_range = random_range(0, CGA_ROWS) as f32;
        return rand_within_range;
    }

    /// Returns a random number within the y range
    pub fn randomize_movement_direction(&mut self) {
        if random_range(0, 2) == 0{
            self.flip_x();
        }
        if random_range(0, 2) == 0{
            self.flip_y();
        }
    }
    
    fn increase_speed(&mut self) {
        self.movement_x = self.movement_x * BALL_SPEEDUP_MULTIPLICATOR;
        self.movement_y = self.movement_y * BALL_SPEEDUP_MULTIPLICATOR;
    }
}