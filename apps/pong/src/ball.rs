use core::ops::ControlFlow;

//use crate::devices::{cga::{CGA_COLUMNS, CGA_ROWS}, pit};
use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};
use usrlib::user_api::usr_get_system_time;
use usrlib::user_cga::Color;
use crate::geometrics::{Line, Point};
use crate::player::{self, Player};
use crate::sound_fx;
use crate::upgrades::upgrade::Upgrade;
use crate::upgrades::upgrade_manager::UpgradeManager;
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
    pub movement_direction: Point,
    pub speed: f32,
    pub last_hit_player: u8
}


impl Ball {
    /// Creates a new ball
    pub const fn new(x: f32, y: f32, start_player: u8) -> Ball {
        Ball {
            object: Point::new(x, y),
            symbol: BALL_SYMBOL,
            color: BALL_COLOR,
            min_x: 0.0,
            max_x: (CGA_COLUMNS -1 ) as f32,
            min_y: 0.0, 
            max_y: (CGA_ROWS-1) as f32,
            movement_direction: Point::new(0.,0.),
            speed: 0.,
            last_hit_player: start_player
        }
    }

    /// Move the ball by one step, flipping the direction if colliding with other object.
    /// Movement direction is definded in `movement_x` and `movement_y`.
    pub fn move_step(&mut self, mut player_1: &mut Player, mut player_2: &mut Player, mut upgrade_manager: &mut UpgradeManager)
    {
        self.check_collisions(player_1,player_2,upgrade_manager);
        self.move_in_movement_direction();
    }

    fn check_collisions(&mut self, mut player_1: &mut Player, mut player_2: &mut Player, mut upgrade_manager: &mut UpgradeManager) {
        // Calculate Trajectory
        let trajectory=  self.get_trajectory();

        // Only check for one collision (early return)
        if self.check_collision_player(player_1, &trajectory) { 
            self.last_hit_player = 1;
            return; 
        }
        if self.check_collision_player(player_2, &trajectory) { 
            self.last_hit_player = 2;
            return; 
        }
        if self.check_collision_borders(&trajectory) { return; }
        if self.check_collision_upgrade(upgrade_manager, player_1, player_2, &trajectory) { return; }
    }

    fn move_in_movement_direction(&mut self) {
        self.object = self.object + self.movement_direction*self.speed;
    }

    fn check_collision_player(&mut self, player: &mut Player, trajectory: &Line) -> bool {
        let hitpoints = player.object.collides_with_line(trajectory);
        if let Some(hitpoint) = hitpoints.first().unwrap() {
            // New Direction depends on where on the bar the ball hits. Middle -> (1,0), Upper side -> (0.2, -0.8) and so on
            let player_to_hitpoint_line = Line::new(player.object.pivot, hitpoint.clone());
            let mut new_direction = player_to_hitpoint_line.to_directional_vector();
            
            self.set_movement_direction(new_direction);
            self.increase_speed();
            sound_fx::play_collision_player();
            return true;
        }
        false
    }

    fn check_collision_borders(&mut self, trajectory: &Line) -> bool {
        // Check collision with border
        let next_y = trajectory.end.y;
        if next_y > self.max_y || next_y < self.min_y {
            self.flip_y();
            sound_fx::play_collision_border();
            return true;
        }
        false
    }
    
    fn get_trajectory(&mut self) -> Line {
        let current_position = self.object;
        let movement_vector = self.movement_direction * self.speed;
        let next_position = current_position + movement_vector;
        Line::new(self.object, next_position)
    }
    
    /// Sets `movement_x` and `movement_y`. Move will be fulfilled after calling `move_step()`.
    pub fn set_movement_direction(&mut self, mut new_direction: Point)
    {
        // Increase x movement to make the game more dynamic.
        new_direction.x = new_direction.x*10.0;
        let mut scaled_direction = new_direction.unit_vector();

        // Secure minimum x movement to prevent boring straight vertical movement
        scaled_direction.y = scaled_direction.y.clamp(-0.3, 0.3);


        let scaled_direction = scaled_direction.unit_vector();

        self.movement_direction = scaled_direction;
    }

    /// Sets `movement_x` and `movement_y`. Move will be fulfilled after calling `move_step()`.
    pub fn increase_speed(&mut self)
    {
        self.speed *= BALL_SPEEDUP_MULTIPLICATOR;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Sets `x` and `y`.
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.object.y = y;
        self.object.x = x;
    }

    /// Checks if ball would clip inside a border/object in the next movement step.
    /// If that would happen, flip the movement.
    #[deprecated]
    fn check_collision(&mut self, player_1: &mut Player, player_2: &mut Player) {
        // Check collision with border
        let next_y = self.object.y + self.movement_direction.y*self.speed;
        if next_y > self.max_y || next_y < self.min_y {
            self.flip_y();
            sound_fx::play_collision_border();
        }

        // Check collition with bar
        let next_x = (self.object.x + self.movement_direction.x*self.speed) as f32;
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
        self.movement_direction.y = -self.movement_direction.y;
    }

    /// Invert `movement_x`
    fn flip_x(&mut self) {
        self.movement_direction.x = -self.movement_direction.x;
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
    
    fn check_collision_upgrade(&mut self, upgrade_manager: &mut UpgradeManager, mut player_1: &mut Player, mut player_2: &mut Player, trajectory: &Line) -> bool {
        let hitpoint = upgrade_manager.instantiated_upgrade.object.collides_with_line(trajectory);
        if let Some(hitpoint) = hitpoint.first().unwrap() {
            // Select players
            let collecting_player;
            let other_player;
            if self.last_hit_player == 1 {
                collecting_player = player_1;
                other_player = player_2;
            } else {
                collecting_player = player_2;
                other_player = player_1;
            }

            upgrade_manager.apply(collecting_player, other_player, self);
            sound_fx::play_collect_upgrade();
            return true;
        }
        false
    }
}