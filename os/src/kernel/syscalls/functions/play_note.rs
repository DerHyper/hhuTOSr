use crate::devices::pcspk;


pub extern "C" fn sys_play_note(frequency: usize, duration_ms: usize) {
    pcspk::SPEAKER.lock().play(frequency, duration_ms);
}
