use crate::devices::cga;
use crate::kernel::coroutines::coroutine::{self, Coroutine};

fn coroutine_loop(coroutine: &mut Coroutine) {
    for i in 0..5 {
        println!("Coroutine {}: iteration {}", coroutine.get_id(), i);
        coroutine.switch(); // Switch to the next coroutine
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

    // Start the first coroutine
    coroutine1.start();
}