extern crate alloc;
use alloc::vec::Vec;
use crate::user_api::{usr_get_char, usr_get_key_queue};


/// Wait for a key press and return the character if it is a valid ASCII character.
pub fn getch() -> char {
   usr_get_char()
}

/// Get all keys from the keyboard buffer and return them as a vector of characters.
pub fn get_all_keys() -> Vec<char> {
   let mut buf: [char; 256] = ['\0'; 256];
   let count = usr_get_key_queue(&mut buf);
   buf[..count].to_vec()
}

/// Wait for the Enter key to be pressed.
pub fn wait_for_return() {
   loop {
      if getch() == '\r' {
         break;
      }
   }
}