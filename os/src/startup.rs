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
#![feature(box_as_ptr)]

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
use core::usize;

use devices::cga; // shortcut for cga
use devices::cga_print; // used to import code needed by println! 
use devices::keyboard; // shortcut for keyboard

use kernel::cpu;
use kernel::interrupts::intdispatcher;

use usrlib::allocator;

use devices::pci::get_pci_bus;
use crate::consts::HEAP_SIZE;
use crate::consts::PAGE_FRAME_SIZE;
use crate::devices::lfb::init_lfb;
use crate::devices::pci::Command;
use crate::devices::pit;
use crate::kernel::cpu::IoPort;
use crate::kernel::interrupts::idt;
use crate::kernel::interrupts::pic;
use crate::kernel::multiboot;
use crate::kernel::multiboot::FramebufferType;
use crate::kernel::multiboot::MultibootInfo;
use crate::kernel::paging::frames;
use crate::kernel::paging::pages::init_kernel_tables;
use crate::kernel::paging::pages::write_cr3;
use crate::library::input;
use crate::user::aufgabe7::graphic_demo;
use crate::user::aufgabe7::pong;
use crate::user::aufgabe10::PfListTest;
use crate::user::aufgabe11::page_table_demo;
use crate::user::aufgabe11::page_table_thread_demo;
use crate::user::aufgabe12::user_app_demo;
use crate::user::aufgabe12::user_app_demo_pid;
use crate::user::aufgabe13::user_app_dump_vmas;
use crate::user::aufgabe13::user_app_fibonacci;

use user::aufgabe1::text_demo;
use user::aufgabe1::keyboard_demo;
use user::aufgabe2::heap_demo;
use user::aufgabe2::sound_demo;
use user::aufgabe4::coroutine_demo;
use user::aufgabe4::queue_demo;
use user::aufgabe4::thread_demo;
use user::aufgabe4::hello_world_thread;
use user::aufgabe5::thread_demo_preemptive;
use user::aufgabe6::thread_demo_timed;
use user::aufgabe8::user_threads;
use user::aufgabe9::syscall_demo;

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
    //hello_world_thread::run();
    thread_demo::run();
}

fn aufgabe5() {
    thread_demo_preemptive::run();
}

fn aufgabe6() {
    thread_demo_timed::run();
}

#[unsafe(no_mangle)]
pub extern "C" fn startup(multiboot_info: &MultibootInfo) {
    kprintln!("Welcome to hhuTOS!");

    // Copy multiboot into on stack, because it lies in physical memory that might get reused after initializing the physical memory allocator
    multiboot::MULTIBOOT_INFO.call_once(|| *multiboot_info);
    kprintln!("Initializing physical memory allocator");
    multiboot::MULTIBOOT_INFO.get().unwrap().init_phys_memory_allocator();

    let heap_addr = unsafe { frames::FRAME_ALLOCATOR.lock().alloc_block(HEAP_SIZE/PAGE_FRAME_SIZE).unwrap().raw() } as usize;
    allocator::init(heap_addr, HEAP_SIZE); // Init memory management
    cga::CGA.lock().clear(); // Bildschirm loeschen
    idt::get_idt().load(); // Load Interrupt Descriptor Table
    pic::PIC.lock().init(); // Init Programmable Interrupt Controller
    intdispatcher::INT_VECTORS.lock().init(); // Init Interrupt Vector Map
    cpu::enable_int(); // Enable interrupts
    keyboard::plugin(); // Init keyboard
    pit::plugin(); // Init PIT

    kprintln!("Scanning PCI bus");
    for device in get_pci_bus().iter() {
        kprintln!("Found PCI device {:04x}:{:04x}", device.read_vendor_id(), device.read_device_id());
    }

    // Just a short demo to show how to access PCI devices
    // For more information, see the OsDev Wiki: https://wiki.osdev.org/PCI, https://wiki.osdev.org/RTL8139
    let rtl8139 = get_pci_bus().iter().find(|device| {
        device.read_vendor_id() == 0x10ec && device.read_device_id() == 0x8139
    });

    if let Some(rtl8139) = rtl8139 {
        kprintln!("Found Realtek RTL8139 network controller");

        // Read the I/O base address from BAR0
        let bar0 = rtl8139.read_bar(0);
        if bar0 & 0x1 == 0 {
            // The address in BAR0 is a 32-bit memory-mapped I/O address.
            // This means that the registers are accessed via memory addresses instead of I/O ports.
            // The card emulated by QEMU uses 16-bit I/O ports,
            // so this code path is never executed in QEMU and is just here as a showcase.
            let mmio_base = bar0 & 0xfffffff0;
            kprintln!("RTL8139 MMIO base address: 0x{:x}", mmio_base);

            // Enable MMIO access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::MemEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            // MMIO access is done via volatile reads to ensure the compiler does not optimize them away
            let mac_address_ptr = (mmio_base) as *const u8;
            let mac_address = unsafe {[
                mac_address_ptr.add(0).read_volatile(),
                mac_address_ptr.add(1).read_volatile(),
                mac_address_ptr.add(2).read_volatile(),
                mac_address_ptr.add(3).read_volatile(),
                mac_address_ptr.add(4).read_volatile(),
                mac_address_ptr.add(5).read_volatile()
            ]};
            kprintln!("MAC address: {:x?}", mac_address);
        } else {
            // The address in BAR0 is a 16-bit I/O port address
            let io_base = (bar0 & 0xfffc) as u16;
            kprintln!("RTL8139 I/O base address: 0x{:x}", io_base);

            // Enable I/O access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::IoEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            let mac_address = unsafe {[
                IoPort::new(io_base + 0).inb(),
                IoPort::new(io_base + 1).inb(),
                IoPort::new(io_base + 2).inb(),
                IoPort::new(io_base + 3).inb(),
                IoPort::new(io_base + 4).inb(),
                IoPort::new(io_base + 5).inb()
            ]};
            kprintln!("MAC address: {:x?}", mac_address);
        }
    }


    // Check the framebuffer type and either show the CGA menu or initialize the linear framebuffer (LFB)
    if let Some(framebuffer_info) = multiboot_info.get_framebuffer_info() {
        match framebuffer_info.typ {
            FramebufferType::Indexed => {
                panic!("Color palette framebuffer not supported!");
            }
            FramebufferType::RGB => {
                init_lfb(
                    framebuffer_info.addr as *mut u8,
                    framebuffer_info.pitch,
                    framebuffer_info.width,
                    framebuffer_info.height,
                    framebuffer_info.bpp
                );

                graphic_demo::run();
            }
            FramebufferType::Text => {
                show_startscreen()
            }
        }
    } else {
        // No framebuffer info available -> Probably CGA mode
        show_startscreen()

    }

    loop{}
}

fn show_startscreen(){
    // TODO: Remove and add to methods, once kernel Methods work
    // syscall_demo::syscall_test();
    // PfListTest::run();
    // page_table_demo::run();
    // page_table_thread_demo::run();
    // user_app_demo::run();
    // user_app_demo_pid::run();
    // user_app_dump_vmas::run();
    user_app_fibonacci::run();

    // print_startscreen();

    // let methods = [
    //     text_demo::run, 
    //     sound_demo::run,
    //     keyboard_demo::run,
    //     thread_demo_preemptive::run,
    //     heap_demo::run,
    //     thread_demo_timed::run,
    //     pong::run
    //     ];

    // // Wait for key press
    // loop{

    //     // Check if number
    //     let input = input::getch();
    //     let index = input.to_digit(10);
    //     if index.is_none() {
    //         println!("{} not a number.", input);
    //         continue;
    //     }

    //     // Check if within range
    //     let index = index.unwrap() as usize;
    //     if index < 1 || index > methods.len() {
    //         println!("{} not within range.", index);
    //         continue;
    //     }

    //     // Clean screen and call Method
    //     { cga::CGA.lock().clear(); }
    //     methods[index-1]();

    //     // Return
    //     println!("\nPress any key to continue.");
    //     input::getch();
    //     print_startscreen();
    // }
}

fn print_startscreen() {
    { cga::CGA.lock().clear(); }
    println!("Welcome to hhuTOS!");
    println!("\n       _~^~^~_\n   \\) /  o o  \\ (/\n     \'_   v   _\'\n     / \'-----\' \\\n");
    println!("1 - Text demo ");
    println!("2 - Sound demo ");
    println!("3 - Keyboard demo ");
    println!("4 - Thread demo ");
    println!("5 - Memory demo ");
    println!("6 - Mutex demo ");
    println!("7 - Pong demo ");
    println!("8 - Kernel vs. User Threads ");
    println!("");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("Panic: {}", info);
    //	kprintln!("{:?}", Backtrace::new());
    loop {}
}

