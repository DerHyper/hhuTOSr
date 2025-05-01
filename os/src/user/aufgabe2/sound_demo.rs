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