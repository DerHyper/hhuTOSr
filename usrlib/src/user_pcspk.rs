use crate::user_api::usr_play_note;


/// All 16 CGA colors.
#[repr(usize)] // store each enum variant as an u8
#[derive(Clone, Copy)]
pub enum PitchNote {
    C0 = 130.81 as usize,
    C0X = 138.59 as usize,
    D0 = 146.83 as usize,
    D0X = 155.56 as usize,
    E0 = 164.81 as usize,
    F0 = 174.61 as usize,
    F0X = 185.00 as usize,
    G0 = 196.00 as usize,
    G0X = 207.65 as usize,
    A0 = 220.00 as usize,
    A0X = 233.08 as usize,
    B0 = 246.94 as usize,

    C1 = 261.63 as usize,
    C1X = 277.18 as usize,
    D1 = 293.66 as usize,
    D1X = 311.13 as usize,
    E1 = 329.63 as usize,
    F1 = 349.23 as usize,
    F1X = 369.99 as usize,
    G1 = 391.00 as usize,
    G1X = 415.30 as usize,
    A1 = 440.00 as usize,
    A1X = 466.16 as usize,
    B1 = 493.88 as usize,

    C2 = 523.25 as usize,
    C2X = 554.37 as usize,
    D2 = 587.33 as usize,
    D2X = 622.25 as usize,
    E2 = 659.26 as usize,
    F2 = 698.46 as usize,
    F2X = 739.99 as usize,
    G2 = 783.99 as usize,
    G2X = 830.61 as usize,
    A2 = 880.00 as usize,
    A2X = 923.33 as usize,
    B2 = 987.77 as usize,
    C3 = 1046.50 as usize,
}

pub fn play_note(note: PitchNote, duration_ms: usize) {
    let frequency = note as usize;
    play(frequency, duration_ms);
}

pub fn play(frequency: usize, duration_ms: usize) {
    usr_play_note(frequency, duration_ms);
}
