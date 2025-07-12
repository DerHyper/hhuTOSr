use crate::{devices::cga::{CGA_COLUMNS, CGA_ROWS}, user::aufgabe7::player::Player};
use crate::user::aufgabe7::sound_fx;

pub struct Ball {
    pub x: u16,
    pub y: u16,
    //pub size: u16,
    pub min_x: u16,
    pub max_x: u16,
    pub min_y: u16,
    pub max_y: u16,
    pub movement_x: i16,
    pub movement_y: i16
}

impl Ball {
    pub const fn new(x: u16, y: u16) -> Ball {
        Ball {
            x: x, 
            y: y, 
            min_x: 0,
            max_x: (CGA_COLUMNS -1 ) as u16,
            min_y: 0, 
            max_y: (CGA_ROWS-1) as u16,
            movement_x: 0,
            movement_y: 0
        }
    }

    pub fn move_step(&mut self, mut player_1: &mut Player, mut player_2: &mut Player)
    {
        self.check_collision(&mut player_1, &mut player_2);
        self.y = (self.y as i16 + self.movement_y) as u16;
        self.x = (self.x as i16 + self.movement_x) as u16;
    }

    pub fn set_movement(&mut self, new_x: i16, new_y: i16)
    {
        self.movement_x = new_x;
        self.movement_y = new_y;
    }

    pub fn set_position(&mut self, x: u16, y: u16) {
        self.y = y;
        self.x = x;
    }

    fn check_collision(&mut self, player_1: &mut Player, player_2: &mut Player) {
        // Check collision with border
        let next_y = (self.y as i16 + self.movement_y) as u16;
        if next_y >= self.max_y {
            self.flip_y();
            sound_fx::play_collision_border();
        }

        // Check collition with bar
        let next_x = (self.x as i16 + self.movement_x) as u16;
        let collides_with_player =
            player_1.is_colliding(next_x, next_y) ||
            player_2.is_colliding(next_x, next_y);
        if collides_with_player {
            self.flip_x();
            sound_fx::play_collision_player();
        }
    }
    
    fn flip_y(&mut self) {
        self.movement_y = -self.movement_y;
    }

    fn flip_x(&mut self) {
        self.movement_x = -self.movement_x;
    }
}