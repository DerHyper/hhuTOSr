use crate::devices::pcspk;

pub fn run() {
 
   let mut speaker = pcspk::SPEAKER.lock();

   speaker.on();
 
}
