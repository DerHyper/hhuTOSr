use usrlib::user_cga::Color;

extern crate alloc;

use alloc::boxed::Box;
use crate::geometrics::Rect;

pub trait UpgradeTypeTrait {
    fn create(&self) -> Box<dyn UpgradeTypeTrait>;
    fn get_symbol(&self) -> char;
    fn get_color(&self) -> Color;
    fn apply_upgrade(&self, collecting_player: &mut crate::player::Player, other_player: &mut crate::player::Player, ball: &mut crate::ball::Ball);
}

pub enum UpgradeType {
    LongerBar,
    AddBall,
    SlowerBalls,
    Blocker
}

pub struct Upgrade {
    pub upgrade_type: Box<dyn UpgradeTypeTrait>,
    pub object: Rect
}

impl Upgrade {
    pub fn new(upgrade_type: Box<dyn UpgradeTypeTrait>, object: Rect) -> Upgrade {
        Upgrade {
            upgrade_type,
            object
        }
    }

    pub fn apply(&self, collecting_player: &mut crate::player::Player, other_player: &mut crate::player::Player, ball: &mut crate::ball::Ball) {
        self.upgrade_type.apply_upgrade(collecting_player, other_player, ball);
    }
}