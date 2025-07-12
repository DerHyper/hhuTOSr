use crate::devices::pcspk;

pub fn play_collision_border() {
    let mut speaker = pcspk::SPEAKER.lock();
    speaker.play(pcspk::C0, 10);
}

pub fn play_collision_player() {
    let mut speaker = pcspk::SPEAKER.lock();
    speaker.play(pcspk::A0, 10);
}

pub fn play_score_point() {
    let mut speaker = pcspk::SPEAKER.lock();
    speaker.play(pcspk::C1, 10);
    speaker.play(pcspk::D1, 7);
}