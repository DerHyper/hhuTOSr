/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: pit                                                             ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Programmable Interval Timer.                                    ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author:  Michael Schoettner, HHU, 15.6.2023                             ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use alloc::boxed::Box;
use core::arch::asm;
use core::sync::atomic::AtomicUsize;
use spin::Once;
use crate::devices::cga;
use crate::devices::cga::{Color, CGA, CGA_COLUMNS, CGA_ROWS};
use crate::kernel::cpu;
use crate::kernel::cpu::IoPort;
use crate::kernel::interrupts::{intdispatcher, pic};
use crate::kernel::interrupts::intdispatcher::InterruptVector;
use crate::kernel::interrupts::isr::ISR;
use crate::kernel::interrupts::pic::Irq;
use crate::kernel::threads::scheduler::{get_scheduler, Scheduler};
use crate::kernel::threads::scheduler;

// Ports
const PORT_CTRL: u16 = 0x43;
const PORT_DATA0: u16 = 0x40;

const TIMER_FREQ: usize = 1193182; // Timer frequency in Hz
const NANOSECONDS_PER_TICK: usize = 1_000_000_000 / TIMER_FREQ; // Nanoseconds per timer tick

/// Global timer instance.
/// Not accessible from outside the module.
/// To get the current system time, use `get_system_time()`.
static TIMER: Once<Timer> = Once::new();

/// Global system time in milliseconds.
static SYSTEM_TIME: AtomicUsize = AtomicUsize::new(0);

/// Characters used for the spinner animation.
static SPINNER_CHARS: &[char] = &['|', '/', '-', '\\'];
static mut SPINNER_CHAR_CURRENT: char = '\\';
const SPINNER_INTERVAL: usize = 250;

/// Get the current system time in milliseconds.
pub fn get_system_time() -> usize {
    SYSTEM_TIME.load(core::sync::atomic::Ordering::Relaxed)
}

/// Wait for a specified number of milliseconds using the system time.
pub fn wait(ms: usize) {
    let start_time = get_system_time();
    let mut current_waiting_time :usize = 0;
    while current_waiting_time < ms {
        let current_time = get_system_time();
        current_waiting_time = current_time - start_time;
    }

}

/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Interrupt service routine implementation.                               ║
   ╚═════════════════════════════════════════════════════════════════════════╝ */

/// Register the timer interrupt handler.
pub fn plugin() {

    let mut pic = pic::PIC.lock(); // Get PIC
    pic.allow(pic::Irq::Timer); // Allow IRQ (PIT)

    // Register TimerISR in intdispatcher
    let timer_isr = Box::new(TimerISR{ interval_ms: 1 });
    intdispatcher::INT_VECTORS.lock().register(intdispatcher::InterruptVector::Pit, timer_isr); 

    // Set Timer Speed to 1 ms
    // 7-6 = Chanal, 5-4 = Access Mode, 3-1 = Operating Mode, 0 = Binary Mode
    // Chanal 0    , Low+High         , Mode 3 (Square)     , 16 Bit
    let command = 0b00_11_011_0;
    let ms_per_tick = TIMER_FREQ/1000;
    unsafe {
        IoPort::new(PORT_CTRL).outb(command);
        IoPort::new(PORT_DATA0) .outb( (ms_per_tick & 0xFF) as u8); // Low
        IoPort::new(PORT_DATA0) .outb((ms_per_tick >> 8) as u8); // High
    }

    // Init Timer
    TIMER.call_once(|| {Timer { 
        control_port: IoPort::new(PORT_CTRL), 
        data_port0: IoPort::new(PORT_DATA0) 
    }});

}

/// The timer interrupt service routine.
struct TimerISR {
    /// The interval between timer interrupts in milliseconds.
    interval_ms: usize,
}

impl ISR for TimerISR {
    fn trigger(&self) {
        // Unlock INT_VECTORS mutex to allow other interrupts
        unsafe { intdispatcher::INT_VECTORS.force_unlock() }

        //kprintln!("   pit::trigger called! {}", get_system_time());
        // Increment System Time 
        SYSTEM_TIME.fetch_add(1, core::sync::atomic::Ordering::SeqCst); // Thread access is ordered with Sequential Consistenz (Very Strong)
        let x = SPINNER_CHARS[1];
        
        // Check for spinner update  
        if get_system_time() % SPINNER_INTERVAL == 0 {
            update_spinner();
        } else {
            return // No Update needed
        }

        scheduler::get_scheduler().yield_cpu();
    }
}

fn update_spinner() {
    // Iterrate over SPINNER_CHARS
    let current_index = SPINNER_CHARS.iter().position(|&x| unsafe{x == SPINNER_CHAR_CURRENT}).unwrap();
    let mut next_index = current_index + 1;
    if current_index >= SPINNER_CHARS.iter().count()-1 {
        next_index = 0;
    }

    let next_spinner_symbol = SPINNER_CHARS[next_index];
    unsafe {SPINNER_CHAR_CURRENT = SPINNER_CHARS[next_index]};

    // Check for cga Access
    let cga = CGA.try_lock();
    if let Some(mut cga) = cga {

        // Set Rotation Symbol
        let pos = cga.getpos();
        cga.print_byte_at_nowrapping(next_spinner_symbol as u8, CGA_COLUMNS-1, 0);
        cga.setpos(pos.0, pos.1);
    }
}

/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Implementation of the PIT driver itself.                                ║
   ╚═════════════════════════════════════════════════════════════════════════╝ */

/// Represents the programmable interval timer.
struct Timer {
    control_port: IoPort,
    data_port0: IoPort
}

impl Timer {
    /// Create a new Timer instance.
    pub const fn new() -> Timer {
        Timer {
            control_port: IoPort::new(PORT_CTRL),
            data_port0: IoPort::new(PORT_DATA0)
        }
    }

    /// Set the timer interrupt interval in milliseconds.
    pub fn set_interrupt_interval(&mut self, interval_ms: usize) {

        /* Hier muss Code eingefuegt werden */

    }
}
