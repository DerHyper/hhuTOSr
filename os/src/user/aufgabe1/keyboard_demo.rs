use crate::devices::cga as cga; // shortcut for cga
use crate::devices::cga_print; // used to import code needed by println! 
use crate::devices::key as key; // shortcut for key
use crate::devices::keyboard as keyboard; // shortcut for keyboard


pub fn run() {

    /* Hier muss Code einfgeügt werden */
    let mut cga = cga::CGA.lock();

    loop{
        let keyboard = keyboard::KEYBOARD.lock();
        let key = keyboard.key_hit();
    }
    // 'key_hit' aufrufen und Zeichen ausgeben

}
