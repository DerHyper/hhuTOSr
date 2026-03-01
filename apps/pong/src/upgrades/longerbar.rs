use usrlib::user_cga::Color;

use crate::upgrades::upgrade::UpgradeTypeTrait;

pub const LONGER_BAR_UPGRADE_SYMBOL: char = 0xBA as char; // '║'
pub const LONGER_BAR_UPGRADE_COLOR: Color = Color::LightGreen;

pub struct LongerBarUpgrade {
    pub symbol: char,
    pub color: Color,
}

impl LongerBarUpgrade {
    pub const fn new() -> LongerBarUpgrade {
        LongerBarUpgrade {
            symbol: LONGER_BAR_UPGRADE_SYMBOL,
            color: LONGER_BAR_UPGRADE_COLOR
        }
    }
}

impl UpgradeTypeTrait for LongerBarUpgrade {
    fn get_symbol() -> char {
        LONGER_BAR_UPGRADE_SYMBOL
    }

    fn get_color() -> Color {
        LONGER_BAR_UPGRADE_COLOR
    }

    fn apply_upgrade(target_player: &mut crate::player::Player, _other_player: &mut crate::player::Player, _ball: &mut crate::ball::Ball) {
        target_player.object.height = target_player.object.height + 1.0;
    }
}