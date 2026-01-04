use alloc::boxed::Box;
use core::arch::asm;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::kernel::cpu;
use crate::kernel::threads::scheduler::{self, get_scheduler, Scheduler};
use crate::kernel::threads::thread::Thread;
use crate::library::queue::LinkedQueue;
use usrlib::spinlock::Spinlock;

/// A more sophisticated lock implementation than `Spinlock`, that blocks waiting threads
/// when the lock is already held. This improves performance, as no time is wasted by threads
/// spinning in a loop while waiting for the lock to be released.
pub struct Mutex<T> {
    /// The lock is represented by an atomic boolean that indicates whether the lock is held.
    lock: AtomicBool,
    /// The data protected by the mutex, stored in an `UnsafeCell` to allow mutable access.
    /// See `Spinlock` for more details on why we use `UnsafeCell`.
    data: UnsafeCell<T>,
    /// A queue of threads waiting for the lock to be released.
    wait_queue: Spinlock<LinkedQueue<Box<Thread>>>
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}
unsafe impl<T> Send for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Mutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
            wait_queue: Spinlock::new(LinkedQueue::new())
        }
    }
    
    /// Try to acquire the lock once without blocking.
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {

        if self.is_locked() {
            return None
        }

        // Lock
        self.lock.swap(true, Ordering::SeqCst);
        let mutex_lock = MutexGuard { lock: self };
        Some(mutex_lock)
    }

    /// Acquire the lock, blocking if necessary until it is available.
    /// This method will dequeue the current thread from the scheduler if the lock is already held
    /// and store it in the `wait_queue`.
    /// Once the lock is available, the next thread in the `wait_queue` will be woken up
    /// so it can try to acquire the lock again.
    pub fn lock(&self) -> MutexGuard<T> {
        
        // Check in scheduler initialized, if not, spinlock
        if !scheduler::get_scheduler().is_initialized() {
            // Bussy-Polling
            while self.is_locked() { 
                unsafe{ asm!("pause"); }
            }

            // Lock
            self.lock.store(true, Ordering::Release);
            return MutexGuard { lock: self };
        }

        // Mutex
        if self.is_locked() {
            // Dequeue current Thread from scheduler and add it to wait_queue
            let mut blocked_thread = scheduler::get_scheduler().prepare_block();
            let thread_ptr = Box::as_mut_ptr(&mut blocked_thread.0);
            { 
                self.wait_queue.lock().enqueue(blocked_thread.0);
            } // Own scope because switch_from_blocked_thread might not finish before new lock() call

            // Switch to next thread in scheduler
            unsafe {
                scheduler::get_scheduler().switch_from_blocked_thread(thread_ptr, blocked_thread.1);
            }
        }

        // Thread Waiting
        self.lock.store(true, Ordering::Release);
        MutexGuard { lock: self }
    }
    
    /// Check if the lock is currently held.
    pub fn is_locked(&self) -> bool {

        // Check if locked using strict ordering load on Atomic Bool
        return self.lock.load(Ordering::Acquire);
    }

    /// Check if the wait queue is currently locked.
    pub fn is_queue_locked(&self) -> bool {
        self.wait_queue.is_locked()
    }

    /// Unlock the mutex, allowing other threads to acquire it.
    /// If there are threads waiting for the lock, the next thread in the wait queue is woken up.
    pub fn unlock(&self) {

        // If thread is waiting for lock, add it back to the scheduler
        if let Some(next_thread) = self.wait_queue.lock().dequeue() {
            scheduler::get_scheduler().ready(next_thread);
        
        // If no thread is waiting, unlock
        } else {
            self.lock.store(false, Ordering::SeqCst);
        }

    }
    
    /// Forcefully unlock the mutex without waking up any waiting threads.
    /// This should only be used in exceptional cases.
    pub unsafe fn force_unlock(&self) {

        self.lock.store(false, Ordering::SeqCst);

    }
}

/// A guard that provides access to the data protected by the mutex.
/// It implements `Deref` and `DerefMut` to allow transparent access to the data.
/// It also implements `Drop` to automatically unlock the mutex when it goes out of scope.
pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.lock.data.get().as_ref().unwrap()
        }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.lock.data.get().as_mut().unwrap()
        }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
