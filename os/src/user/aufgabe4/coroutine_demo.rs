use crate::devices::cga;
use crate::kernel::coroutines::coroutine::{self, Coroutine};



fn coroutine_loop(coroutine: &mut Coroutine) {
    let mut i = 0;

    // Loop indefinitely, printing the coroutine ID and a counter
    // position on the screen is determined by the coroutine ID
    loop {
        let print_offset = 5 + coroutine.get_id();
        cga::CGA.lock().setpos(5,print_offset);
        print!("Coroutine [{}]: {}", coroutine.get_id(), i);

        coroutine.switch(); // Switch to the next coroutine
        i += 1;
    }
}

pub fn run() {
    // Create 3 coroutines
    let mut coroutine1 = Coroutine::new(coroutine_loop);
    let mut coroutine2 = Coroutine::new(coroutine_loop);
    let mut coroutine3 = Coroutine::new(coroutine_loop);

    // Set the next pointers to form a circular linked list
    coroutine1.set_next(&mut coroutine2);
    coroutine2.set_next(&mut coroutine3);
    coroutine3.set_next(&mut coroutine1);

    // Initialize the CGA device
    cga::CGA.lock().clear(); // Clear the screen
    println!("Coroutine Demo:");

    // Start the first coroutine
    coroutine1.start();
}