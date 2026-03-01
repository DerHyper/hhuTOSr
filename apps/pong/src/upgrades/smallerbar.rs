use usrlib::user_cga::Color;

use crate::upgrades::upgrade::UpgradeTypeTrait;

pub const SMALLER_BAR_UPGRADE_SYMBOL: char = 0xCD as char; // '═'
pub const SMALLER_BAR_UPGRADE_COLOR: Color = Color::LightRed;

pub struct SmallerBarUpgrade {
    pub symbol: char,
    pub color: Color,
}

impl SmallerBarUpgrade {
    pub const fn new() -> SmallerBarUpgrade {
        SmallerBarUpgrade {
            symbol: SMALLER_BAR_UPGRADE_SYMBOL,
            color: SMALLER_BAR_UPGRADE_COLOR
        }
    }
}

impl UpgradeTypeTrait for SmallerBarUpgrade {
    fn get_symbol() -> char {
        SMALLER_BAR_UPGRADE_SYMBOL
    }

    fn get_color() -> Color {
        SMALLER_BAR_UPGRADE_COLOR
    }

    fn apply_upgrade(target_player: &mut crate::player::Player, _other_player: &mut crate::player::Player, _ball: &mut crate::ball::Ball) {
        if target_player.object.height-2.0 > 1.0 {
            target_player.object.height = target_player.object.height - 2.0;
        }
    }
}