//use crate::devices::pcspk;

use usrlib::user_pcspk;

/// Plays a sound using the `pcspk`
pub fn play_collision_border() {
    user_pcspk::play_note(user_pcspk::PitchNote::C0, 10);
}

/// Plays a sound using the `pcspk`
pub fn play_collision_player() {
    user_pcspk::play_note(user_pcspk::PitchNote::A0, 10);
}

/// Plays a sound using the `pcspk`
pub fn play_score_point() {
    user_pcspk::play_note(user_pcspk::PitchNote::C1, 10);
    user_pcspk::play_note(user_pcspk::PitchNote::D1, 7);
}

/// Plays a sound using the `pcspk`
pub fn play_collect_upgrade() {
    user_pcspk::play_note(user_pcspk::PitchNote::C1, 10);
    user_pcspk::play_note(user_pcspk::PitchNote::A1, 5);
}