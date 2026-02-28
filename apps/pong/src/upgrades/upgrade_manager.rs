extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::{geometrics::Rect, upgrades::{longerbar::LongerBarUpgrade, upgrade::{Upgrade, UpgradeTypeTrait}}};
use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};

pub struct UpgradeManager {
    upgrade_types: Vec<Box<dyn UpgradeTypeTrait>>,
    pub instantiated_upgrades: Vec<Upgrade>,
}

impl UpgradeManager {
    pub fn new() -> UpgradeManager {
        let mut upgrade_types = Vec::new();
        upgrade_types.push(Box::new(LongerBarUpgrade::new()));

        UpgradeManager {
            upgrade_types: Vec::new(),
            instantiated_upgrades: Vec::new()
        }
    }

    pub fn register_upgrade_type(&mut self, upgrade_type: Box<dyn UpgradeTypeTrait>) {
        self.upgrade_types.push(upgrade_type);
    }

    pub fn instantiate_random_upgrade(&mut self) {
        let upgrade_type = self.pick_random_upgrade_type();
        if upgrade_type.is_none() {
            return;
        }
        let upgrade_type = upgrade_type.unwrap();

        let upgrade_object = self.pick_random_upgrade_position();
        
        let upgrade = Upgrade {
            upgrade_type: upgrade_type.create(),
            object: upgrade_object
        };

        self.push_upgrade(upgrade);
    }

    pub fn push_upgrade(&mut self, upgrade: Upgrade) {
        self.instantiated_upgrades.push(upgrade);
    }

    pub fn get_instantiated_upgrades(&self) -> &Vec<Upgrade> {
        &self.instantiated_upgrades
    }

    fn pick_random_upgrade_type(&self) -> Option<&Box<dyn UpgradeTypeTrait>> {
        if self.upgrade_types.len() == 0 {
            return None;
        }
        let random_index = crate::utils::random_range(0, self.upgrade_types.len());
        Some(&self.upgrade_types[random_index])
    }

    fn pick_random_upgrade_position(&self) -> Rect {
        let x = crate::utils::random_range(5, CGA_COLUMNS-5 as usize) as f32;
        let y = crate::utils::random_range(0, CGA_ROWS as usize) as f32;

        Rect::new(x, y, 1.0, 1.0)
    }

}