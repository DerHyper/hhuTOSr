use alloc::string::ToString;
use library::queue;

use crate::library;

fn is_odd(u :&u32) -> bool {
    u % 2 != 0
}

pub fn run() {
    println!("Queue Demo: Remove first odd element");

    test_first();
    test_middle();
    test_last();
    test_multible();

}

fn test_first() {
    let mut queue = queue::LinkedQueue::<u32>::new();
    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(2);

    println!("   Queue old: {:?}", queue.to_string());
    queue.remove(is_odd);
    println!("   Queue new: {:?}\n", queue.to_string());
}

fn test_middle() {
    let mut queue = queue::LinkedQueue::<u32>::new();
    queue.enqueue(2);
    queue.enqueue(1);
    queue.enqueue(2);

    println!("   Queue old: {:?}", queue.to_string());
    queue.remove(is_odd);
    println!("   Queue new: {:?}\n", queue.to_string());
}

fn test_last() {
    let mut queue = queue::LinkedQueue::<u32>::new();
    queue.enqueue(2);
    queue.enqueue(2);
    queue.enqueue(1);

    println!("   Queue old: {:?}", queue.to_string());
    queue.remove(is_odd);
    println!("   Queue new: {:?}\n", queue.to_string());
}

fn test_multible() {
    let mut queue = queue::LinkedQueue::<u32>::new();
    queue.enqueue(2);
    queue.enqueue(1);
    queue.enqueue(1);
    queue.enqueue(1);
    queue.enqueue(2);

    println!("   Queue old: {:?}", queue.to_string());
    queue.remove(is_odd);
    println!("   Queue new: {:?}\n", queue.to_string());
}