use usrlib::user_cga::Color;

use crate::upgrades::upgrade::UpgradeTypeTrait;

pub const ADD_BALL_UPGRADE_SYMBOL: char = 0x0F as char; // '☼'
pub const ADD_BALL_UPGRADE_COLOR: Color = Color::LightBlue;

pub struct AddBallUpgrade {
    pub symbol: char,
    pub color: Color,
}

impl AddBallUpgrade {
    pub const fn new() -> AddBallUpgrade {
        AddBallUpgrade {
            symbol: ADD_BALL_UPGRADE_SYMBOL,
            color: ADD_BALL_UPGRADE_COLOR
        }
    }
}

impl UpgradeTypeTrait for AddBallUpgrade {
    fn get_symbol() -> char {
        ADD_BALL_UPGRADE_SYMBOL
    }

    fn get_color() -> Color {
        ADD_BALL_UPGRADE_COLOR
    }

    fn apply_upgrade(target_player: &mut crate::player::Player, _other_player: &mut crate::player::Player, _ball: &mut crate::ball::Ball) {
        // This is being done via BallEvents
    }
}