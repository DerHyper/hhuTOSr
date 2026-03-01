extern crate alloc;

use core::{mem, ptr::null};

use crate::{geometrics::Rect, pong_game, upgrades::{longerbar::LongerBarUpgrade, upgrade::{Upgrade, UpgradeType, UpgradeTypeTrait}}, utils::Timer};
use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};

const UPGRADE_SPAWN_INTERVAL: usize = 10000; // Spawn upgrade every 10 seconds
const UPGRADE_SIZE: f32 = 2.0;

pub struct UpgradeManager {
    pub instantiated_upgrade: Upgrade,
    upgrade_spawn_timer: Timer
}

impl UpgradeManager {
    pub fn new() -> UpgradeManager {
        UpgradeManager {
            instantiated_upgrade: Upgrade {
                upgrade_type: UpgradeType::None,
                object: Rect::new(0.0, 0.0, UPGRADE_SIZE, UPGRADE_SIZE)
            },
            upgrade_spawn_timer: Timer::new()
        }
    }

    /// Called every frame
    pub fn update(&mut self) {
        if self.upgrade_spawn_timer.elapsed() >= UPGRADE_SPAWN_INTERVAL {
            self.instantiate_random_upgrade();
            self.upgrade_spawn_timer.reset();
        }
    }

    /// Instantiates a random upgrade at a random position on the screen and adds it to the list of instantiated upgrades
    pub fn instantiate_random_upgrade(&mut self) {
        let upgrade_type = UpgradeType::get_random();
        let upgrade_object = self.pick_random_upgrade_position();
        
        let upgrade = Upgrade {
            upgrade_type: upgrade_type,
            object: upgrade_object
        };

        self.push_upgrade(upgrade);
    }

    pub fn push_upgrade(&mut self, upgrade: Upgrade) {
        self.instantiated_upgrade = upgrade;
    }

    fn pick_random_upgrade_position(&self) -> Rect {
        let x = crate::utils::random_range(5, CGA_COLUMNS-5 as usize) as f32;
        let y = crate::utils::random_range(0, CGA_ROWS as usize) as f32;

        Rect::new(x, y, UPGRADE_SIZE, UPGRADE_SIZE)
    }

    pub fn get_symbol(&self) -> char {
        self.instantiated_upgrade.upgrade_type.get_symbol()
    }

    pub fn get_color(&self) -> usrlib::user_cga::Color {
        self.instantiated_upgrade.upgrade_type.get_color()
    }
}