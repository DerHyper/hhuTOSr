use crate::devices::cga as cga; // shortcut for cga
use crate::devices::cga_print; // used to import code needed by println! 
use crate::devices::key as key; // shortcut for key
use crate::devices::keyboard as keyboard; // shortcut for keyboard


pub fn run() {

    // Set repeat rate
    {
        let mut keyboard = keyboard::KEYBOARD.lock();
        keyboard.set_repeat_rate(0, 3);
    }

    // 'key_hit' aufrufen und Zeichen ausgeben
    loop{
        let mut keyboard = keyboard::KEYBOARD.lock();
        let key = keyboard.key_hit();
        let invalid_key = Default::default();
        if key != invalid_key {
            print_to_terminal(key);
        }
    }
}

fn print_to_terminal(mut key: key::Key) {
    let mut cga = cga::CGA.lock();
    let value: u8 = key.get_ascii();
    kprintln!("Got Key: {}", value);
    cga.print_byte(value);
}
