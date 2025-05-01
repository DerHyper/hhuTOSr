use crate::devices::pcspk;
use crate::keyboard;
use crate::cga;

pub fn run() {
   println!("Playing Tetris ...");
   pcspk::tetris();
   println!("Playing Aerodynamic by Daft Punk ...");
   pcspk::aerodynamic();
   println!("Done!");
 
}

fn ask_for_input() {
   println!("Press a key to stop the sound");

   // Wait for key press
   let mut keyboard = keyboard::KEYBOARD.lock();
   let invalid_key = Default::default();
   let mut key = keyboard.key_hit();
   while key == invalid_key{
      key = keyboard.key_hit();
   }

   let mut cga = cga::CGA.lock();
   cga.clear();
}