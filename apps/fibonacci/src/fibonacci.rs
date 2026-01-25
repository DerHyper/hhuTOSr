#![no_std]

use core::panic::PanicInfo;
use usrlib::{print, println, user_api::{usr_dump_vmas, usr_hello_world}};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    println!("Fibonacci sequence:");
    let x = BigStruct { 
        x1: 0, x2: 0, x3: 0, x4: 0, x5: 0, 
        x6: 0, x7: 0, x8: 0, x9: 0, x10: 0, 
        x11: 0, x12: 0, x13: 0, x14: 0, x15: 0, 
        x16: 0, x17: 0, x18: 0, x19: 0, x20: 0 } ;
    fibonacci(0,1, x);
    loop {}
}

struct BigStruct {
    x1: usize,
    x2: usize,
    x3: usize,
    x4: usize,
    x5: usize,
    x6: usize,
    x7: usize,
    x8: usize,
    x9: usize,
    x10: usize,
    x11: usize,
    x12: usize,
    x13: usize,
    x14: usize,
    x15: usize,
    x16: usize,
    x17: usize,
    x18: usize,
    x19: usize,
    x20: usize
}

#[warn(unconditional_recursion)]
fn fibonacci(a:usize , b:usize, x:BigStruct)
{
    let a2 = b;
    let b2_option = a.checked_add(b);
    if b2_option.is_none() {
        println!("fibonacci reached overflow, restarting ...");
        fibonacci(0,1,x);
        return; // Will not return, but needed because of x borrow
    }
    let b2 = b2_option.unwrap();

    println!("{}",a);
    fibonacci(a2,b2,x);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Paniced at fibonacci");
    loop {}
}
