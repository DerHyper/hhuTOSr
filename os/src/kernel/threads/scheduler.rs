/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: scheduler                                                       ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: A basic round-robin scheduler for cooperative threads.          ║
   ║         No priorities supported.                                        ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Autor:  Michael Schoettner, 15.05.2023                                  ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Display;
use core::{fmt, panic, ptr};
use core::sync::atomic::AtomicUsize;
use spin::Once;
use crate::library::spinlock::Spinlock as Mutex;
use crate::kernel::threads::idle_thread::idle_thread;
use crate::kernel::threads::thread;
use crate::kernel::threads::thread::Thread;
use crate::library::queue::LinkedQueue;
use crate::kernel::{allocator, cpu};

/// Global scheduler instance
static SCHEDULER: Once<Scheduler> = Once::new();

/// Global access to the scheduler.
pub fn get_scheduler() -> &'static Scheduler {
    SCHEDULER.call_once(|| { Scheduler::new() })
}

/// Unlock the scheduler state.
/// This function is called from assembly code.
/// Usually, the mutex would be unlocked automatically when going out of scope.
/// However, since we switch to a different thread in `yield_cpu()` and `exit()`,
/// the scope is not left and the mutex remains locked.
/// As a workaround, we provide this function to unlock the scheduler manually.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unlock_scheduler() {
    unsafe {
        get_scheduler().state.force_unlock();
    }
}

/// The state of the scheduler.
/// It contains the active thread and the ready queue with all other threads.
/// The state is contained in its own struct so that it can be locked via a mutex.
struct SchedulerState {
    active_thread: Option<Box<Thread>>,
    ready_queue: LinkedQueue<Box<Thread>>,
    initialized: bool,
}

/// Represents the scheduler.
/// It is round-robin-based and uses a queue to manage the threads.
pub struct Scheduler {
    state: Mutex<SchedulerState>,
}

impl Scheduler {
    /// Create a new scheduler instance with an empty ready queue
    /// and an idle thread as the active thread.
    pub fn new() -> Self {
        let state = SchedulerState {
            active_thread: Some(Thread::new_kernel_thread(idle_thread)),
            ready_queue: LinkedQueue::new(),
            initialized: false,
        };
        
        Scheduler { state:  Mutex::new(state) }
    }

    /// Get the ID of the currently active thread.
    pub fn get_active_tid(&self) -> usize {
        let state = self.state.lock();
        
        state.active_thread.as_ref().unwrap().get_id()
    }

    /// Start the scheduler.
    /// This function must only be called once.
    pub fn schedule(&self) {
        let mut state = self.state.lock();
        state.initialized = true;

        // The active thread is never None, since we must at least have the idle thread.
        state.active_thread.as_mut().unwrap().start();
    }

    /// Register a new thread in the ready queue.
    pub fn ready(&self, thread: Box<Thread>) {
        let mut state = self.state.lock();
        
        state.ready_queue.enqueue(thread);
    }

    /// Terminate the current (calling) thread and switch to the next one.
    pub fn exit(&self) {
        let mut state = self.state.lock();

        // The active thread is never None, since we must at least have the idle thread.
        let mut current = state.active_thread.take().unwrap();
        // The idle thread never exits, so there must be at least one thread in the queue.
        let next = state.ready_queue.dequeue().unwrap();
            
        // Set the dequeued thread as the active thread,
        // overwriting the current one, which we want to exit.
        state.active_thread = Some(next);
        
        unsafe {
            // Switch to the next thread.
            // `current` still contains the old thread we want to exit,
            // while `state.active_thread` contains the next one.
            Thread::switch(current.as_mut(), state.active_thread.as_mut().unwrap().as_mut());
        }
    }

    /// Yield the CPU and switch to the next thread in the ready queue.
    pub fn yield_cpu(&self) {

        // Check if state can be locked
        let state = self.state.try_lock();
        if state.is_none() {
            return
        }

        // Check if scheduler was initialized
        let mut state  = state.unwrap();
        if !state.initialized {
            return
        }

        // Check if there is a next thread to switch to.
        // If yes, put current back into queue and pop out the next
        if let Some(mut next_thread) = state.ready_queue.dequeue() {
            
            // check if allocator can be used
            if allocator::is_locked() {
                return
            }

            // active_thread not empty
            if let Some(mut current_thread) = state.active_thread.take() {
                
                // Pointer to thread in Box
                let current_ptr = Box::<Thread>::as_mut_ptr(&mut current_thread); 
                let next_ptr = Box::<Thread>::as_mut_ptr(&mut next_thread);

                // Move Box to Active/Queue, thread does not move
                state.active_thread = Some(next_thread);
                state.ready_queue.enqueue(current_thread); 

                unsafe{
                    Thread::switch(current_ptr, next_ptr)
                };
                

            // active_thread empty
            } else {
                panic!();
            }

        } else { 
            // Currently idle thread
        }

    }

    /// Kill the thread with the given ID by removing it from the ready queue.
    pub fn kill(&self, to_kill_id: usize) {
        let fn_same_id =  |x :&Box<Thread>| (*x).get_id() == to_kill_id;
        self.state.lock().ready_queue.remove(fn_same_id);
    }


    /// Check if the scheduler state is currently locked.
    pub fn is_locked(&self) -> bool {
        self.state.is_locked()
    }

    pub fn is_initialized(&self) -> bool {
        self.state.lock().initialized
    }

    /// Prepare the current thread for blocking.
    /// This functions disables interrupts and return the current thread,
    /// as well as the return value from `cpu::disable_int_nested()`.
    /// To complete the blocking operation call `switch_from_blocked_thread()`,
    /// which will enable interrupts again and resume the scheduler.
    pub fn prepare_block(&self) -> (Box<Thread>, bool) {

        // Disable Interrupts
        let int_was_enabled = cpu::disable_int_nested();

        // Take and return active Thread
        let current_thread = self.state.lock().active_thread.take().unwrap(); 
        return (current_thread, int_was_enabled);

    }

    /// Complete a blocking operation begun with `prepare_block()`.
    /// This resumes the scheduler and switches to the next thread in the ready queue.
    pub unsafe fn switch_from_blocked_thread(&self, blocked_thread: *mut Thread, interrupts_enabled: bool) {
        
        // Get to next Thread in Ready Queue
        let next = self.state.lock().ready_queue.dequeue();
        let mut next_thread;
        match next {
            Some(thread) => next_thread = thread,
            None => next_thread = Thread::new_kernel_thread(idle_thread)
        }

        // Switch to next Thread in Ready Queue
        unsafe {
            Thread::switch(blocked_thread, &mut *next_thread);
        }

        // Enable Interrupts
        cpu::enable_int_nested(interrupts_enabled);
    }
}

impl Display for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state = self.state.lock();
        let active = state.active_thread.as_ref().unwrap();
        
        write!(f, "active: {}, ready: {}", active, state.ready_queue)
    }
}
