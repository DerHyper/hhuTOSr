use core::mem;

use usrlib::user_cga::Color;

use crate::{geometrics::Rect, upgrades::longerbar::LongerBarUpgrade};

pub trait UpgradeTypeTrait {
    fn get_symbol() -> char;
    fn get_color() -> Color;
    fn apply_upgrade(collecting_player: &mut crate::player::Player, other_player: &mut crate::player::Player, ball: &mut crate::ball::Ball);
}

#[derive(PartialEq)]
pub enum UpgradeType {
    None,
    LongerBar,
    // AddBall,
    // SlowerBalls,
    // Blocker
}

impl UpgradeType {
    pub fn apply_upgrade(&self, collecting_player: &mut crate::player::Player, other_player: &mut crate::player::Player, ball: &mut crate::ball::Ball) {
        match self {
            UpgradeType::LongerBar => LongerBarUpgrade::apply_upgrade(collecting_player, other_player, ball),
            UpgradeType::None => {},
            // UpgradeType::AddBall => todo!(),
            // UpgradeType::SlowerBalls => todo!(),
            // UpgradeType::Blocker => todo!(),
        }
    }

    pub fn get_symbol(&self) -> char {
        match self {
            UpgradeType::LongerBar => LongerBarUpgrade::get_symbol(),
            UpgradeType::None => ' ',
            // UpgradeType::AddBall => todo!(),
            // UpgradeType::SlowerBalls => todo!(),
            // UpgradeType::Blocker => todo!(),
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            UpgradeType::LongerBar => LongerBarUpgrade::get_color(),
            UpgradeType::None => Color::Black,
            // UpgradeType::AddBall => todo!(),
            // UpgradeType::SlowerBalls => todo!(),
            // UpgradeType::Blocker => todo!(),
        }
    }

    pub fn from(index: usize) -> UpgradeType {
        match index {
            0 => UpgradeType::None, // No Upgrade
            1 => UpgradeType::LongerBar,
            // 2 => UpgradeType::AddBall,
            // 3 => UpgradeType::SlowerBalls,
            // 4 => UpgradeType::Blocker,
            _ => panic!("Invalid upgrade type index")
        }
    }

    pub fn get_random() -> UpgradeType {
        let number_of_upgrade_types = mem::variant_count::<UpgradeType>();
        if number_of_upgrade_types == 0 {
            panic!("No upgrade types registered");
        }
        let random_index = crate::utils::random_range(1, number_of_upgrade_types);
        UpgradeType::from(random_index)
    }
}

pub struct Upgrade {
    pub upgrade_type: UpgradeType,
    pub object: Rect
}

impl Upgrade {
    pub fn new(upgrade_type: UpgradeType, object: Rect) -> Upgrade {
        Upgrade {
            upgrade_type,
            object
        }
    }

    pub fn apply(&self, collecting_player: &mut crate::player::Player, other_player: &mut crate::player::Player, ball: &mut crate::ball::Ball) {
        self.upgrade_type.apply_upgrade(collecting_player, other_player, ball);
    }
}