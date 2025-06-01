/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: startup                                                         ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Here is the main function called first from the boot code as    ║
   ║         well as the panic handler. All features are set and all modules ║
   ║         are imported.                                                   ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Michael Schoettner, Univ. Duesseldorf, 5.2.2024                 ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
#![no_std]
#![allow(dead_code)] // avoid warnings
#![allow(unused_variables)] // avoid warnings
#![allow(unused_imports)]
#![allow(unused_macros)]
#![feature(abi_x86_interrupt)] // needed for interrupts
#![feature(naked_functions)] // needed for naked functions

extern crate alloc;
extern crate spin; // we need a mutex in devices::cga_print

// insert other modules
#[macro_use] // import macros, too
mod devices;
mod kernel;
mod user;
mod consts;
mod library;

use core::arch::asm;
use core::panic::PanicInfo;

use devices::cga; // shortcut for cga
use devices::cga_print; // used to import code needed by println! 
use devices::keyboard; // shortcut for keyboard

use kernel::cpu;
use kernel::allocator;
use kernel::interrupts::intdispatcher;
use kernel::threads;

use crate::kernel::interrupts::idt;
use crate::kernel::interrupts::pic;

use user::aufgabe1::text_demo;
use user::aufgabe1::keyboard_demo;
use user::aufgabe2::heap_demo;
use user::aufgabe2::sound_demo;
use user::aufgabe4::coroutine_demo;
use user::aufgabe4::queue_demo;
use user::aufgabe4::thread_demo;


fn aufgabe1() {
    text_demo::run();
    println!("");
    keyboard_demo::run();
}

fn aufgabe2() {
    heap_demo::run();
    sound_demo::run();
}

fn aufgabe4() {
    //coroutine_demo::run();
    //queue_demo::run(); // Test the queue implementation
    thread_demo::run();
}

#[unsafe(no_mangle)]
pub extern "C" fn startup() {
    kprintln!("Welcome to hhuTOS!");

    allocator::init(); // Init memory management
    cga::CGA.lock().clear(); // Bildschirm loeschen
    idt::get_idt().load(); // Load Interrupt Descriptor Table
    pic::PIC.lock().init(); // Init Programmable Interrupt Controller
    intdispatcher::INT_VECTORS.lock().init(); // Init Interrupt Vector Map
    cpu::enable_int(); // Enable interrupts
    keyboard::plugin(); // Init keyboard
    threads::scheduler::get_scheduler().schedule();
    
    //aufgabe1();
    //aufgabe2();
    //aufgabe4();

    loop{}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("Panic: {}", info);
    //	kprintln!("{:?}", Backtrace::new());
    loop {}
}

