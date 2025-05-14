use crate::devices::cga as cga; // shortcut for cga
use crate::devices::cga_print; // used to import code needed by println! 
use crate::devices::key as key; // shortcut for key
use crate::devices::keyboard as keyboard; // shortcut for keyboard
use crate::library::input;

pub fn run() {

    // Set repeat rate
    {
        let mut keyboard = keyboard::KEYBOARD.lock();
        keyboard.set_repeat_rate(0, 3);
    }

    // 'key_hit' aufrufen und Zeichen ausgeben
    loop {
        let input = input::getch();
        let mut cga = cga::CGA.lock();
        cga.print_byte(input as u8);
    }
}
