/*
 * Module: thread
 *
 * Description: Contains functions to create, start, switch and end threads.
 *
 * Author: Michael Schoettner, Heinrich Heine University Duesseldorf, 15.05.2023
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 07.08.2025
 */

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{fmt, ptr};
use core::arch::naked_asm;
use core::fmt::Display;
use core::sync::atomic::AtomicUsize;
use crate::consts::{PAGE_SIZE, STACK_ENTRY_SIZE, STACK_SIZE, USER_CODE_VIRT_START, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
use crate::kernel::paging::frames::FRAME_ALLOCATOR;
use crate::kernel::{cpu, multiboot, processes};
use crate::kernel::paging::pages::{self, PageFlags, PageTable, map_user_app, map_user_stack, write_cr3};
use usrlib::user_api::usr_thread_exit;
use usrlib::allocator;
use crate::kernel::threads::scheduler::{get_scheduler, set_current_thread};

unsafe extern "C" {
    fn _tss_set_rsp0(rsp0: usize);
}

static THREAD_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_id() -> usize {
    THREAD_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

/// Low-level routine for starting a thread.
#[unsafe(naked)]
unsafe extern "C" fn thread_start(stack_ptr: usize, next_pml4: usize) {
    naked_asm!(
        "mov rsp, rdi", // Switch stack
        "mov cr3, rsi", // Switch Page Table (pml4)

        "call unlock_scheduler", // Unlock scheduler

        "popf", // Pop rflags
        "pop rbp", // Pop all other registers
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "ret" // Return to 'kickoff'
    )
}

/// Low-level routine for switching to the next thread.
/// `current_stack_ptr` is a pointer to `stack_ptr` of the next coroutine (where the rsp is saved).
/// `next_stack` is the value of `stack_ptr` of the next thread (the new rsp value).
#[unsafe(naked)]
unsafe extern "C" fn thread_switch(current_stack_ptr: *mut usize, next_stack: usize, next_stack_end: usize, next_pml4: usize) {
    naked_asm!(
        // Save all registers of the current thread on its stack
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "pushf",

        // Save stackpointer in 'current_stack_ptr' (first parameter)
        "mov [rdi], rsp",

        // Update TSS rsp0 to 'next_stack_end' (third parameter)
        "mov rdi, rdx", // rdx = next_stack_end
        "call _tss_set_rsp0",

        // Switch stack to 'next_stack' (second parameter)
        "mov rsp, rsi",

        // Switch page table to 'next_pml4' (fourth parameter)
        "mov cr3, rcx",

        // Unlock scheduler
        "call unlock_scheduler",

        // Load all registers of the next thread
        "popf",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",

        "ret"
    )
}

#[unsafe(naked)]
unsafe extern "C" fn thread_user_start(stack_ptr: usize) {
    naked_asm!(
        "mov rsp, rdi", // Switch stack
        "pop rdi",
        "iretq" // Return to user mode
    )
}

/// Represents a coroutine in the system.
/// It contains the kernel and user stacks and the entry function.
/// Threads must be registered in the scheduler and are run automatically
/// once the scheduler is started.
#[repr(C)]
pub struct Thread {
    id: usize,
    is_kernel_thread: bool,
    kernel_stack: Vec<u64>,
    stack_ptr: usize, // Pointer on the stack to the saved context
    entry: fn(),
    pub page_table: &'static mut PageTable,
    process_id: usize,
    pub user_app_size: Option<usize> // Size of the user_app
}

impl Thread {
    /// Create a new thread with the given entry function.
    pub fn new_kernel_thread(entry: fn(), process_id: usize) -> Box<Thread> {
        // Allocate memory for the kernel stack and initialize it to zero
        let mut kernel_stack = Vec::<u64>::with_capacity(STACK_SIZE / 8);
        for _ in 0..kernel_stack.capacity() {
            kernel_stack.push(0);
        }

        // Allocate page table
        let page_table = pages::init_kernel_tables();

        // Allocate memory for the user stack and initialize it to zero
        // Set the stack pointer to the top of the stack
        let stack_ptr = unsafe { map_user_stack(page_table) };

        let user_app_size: Option<_> = None;

        // Create a new thread object
        let mut thread = Box::new(
            Thread { id: next_id(), is_kernel_thread: true, kernel_stack, stack_ptr, entry, page_table, process_id, user_app_size }
        );

        // Prepare the stack for the thread so it can be started via `thread_start()`
        thread.prepare_kernel_stack();
        thread
    }

    pub fn new_user_thread(entry: fn(), process_id: usize) -> Box<Thread> {
        // Allocate memory for the kernel stack and initialize it to zero
        let mut kernel_stack = Vec::<u64>::with_capacity(STACK_SIZE / 8);
        for _ in 0..kernel_stack.capacity() {
            kernel_stack.push(0);
        }

        // Allocate page table
        let page_table = pages::init_kernel_tables();

        // Allocate memory for the user stack and initialize it to zero
        // Set the stack pointer to the top of the stack
        let stack_ptr = unsafe { map_user_stack(page_table) };

        // Map user cga
        unsafe { pages::map_user_cga(page_table) };

        // Get user app from TAR-archive
        let app_name = processes::process::get_app_name(process_id)
            .expect("Process has no app name");

        let archive = multiboot::MULTIBOOT_INFO
            .get()
            .expect("MULTIBOOT_INFO not loaded")
            .get_initrd_archive()
            .expect("No module found");
        let mut app_data: Option<&[u8]> = None;

        for entry in archive.entries() {
            let filename = entry.filename();
            let filename_str :&str = filename.as_str().unwrap();
            kprintln!("Found file: {}", filename_str);
            if filename_str == app_name {
                app_data = Some(entry.data());
                break;
            }
        }

        let app_data = app_data.expect("Application not found in initrd");

        // Allocate physical memory for user app of process_id to memory
        let app_size = app_data.len();
        let num_pages = (app_size + PAGE_SIZE - 1) / PAGE_SIZE;

        // allocate physical memory
        let phys_start = unsafe {
            FRAME_ALLOCATOR
                .lock()
                .alloc_block(num_pages)
                .expect("Failed to allocate app frames")
        };

        // Copy user app to physical memory
        unsafe {
            core::ptr::copy_nonoverlapping(
                app_data.as_ptr(),
                phys_start.as_mut_ptr(),
                app_size,
            );
        }

        // Map physical memory on page_table
        unsafe {
            map_user_app(page_table, num_pages, phys_start);
        }

        // Set entry method 
        let entry: fn() = unsafe {
            core::mem::transmute(USER_CODE_VIRT_START)
        };

        // Size
        let user_app_size: Option<_> = Some(app_size);

        // Create a new thread object
        let mut thread = Box::new(
            Thread { id: next_id(), is_kernel_thread: false, kernel_stack, stack_ptr, entry, page_table, process_id, user_app_size }
        );

        // Prepare the stack for the thread so it can be started via `thread_start()`
        thread.prepare_kernel_stack();
        thread
    }

    /// Start the thread.
    /// This function is only once by the scheduler.
    /// The scheduler does further thread switching via `switch()`.
    pub fn start(&mut self) {

        unsafe {
            write_cr3(self.page_table);
        }

        set_current_thread(self);

        unsafe {
            thread_start(self.stack_ptr, self.page_table as *const PageTable as usize);
        }
    }

    /// Switch from the `current` thread to the `next` thread.
    /// This function is called by the scheduler to switch between threads.
    pub unsafe fn switch(current: *mut Thread, next: *mut Thread) {
        unsafe {
            let current = &mut *current;
            let next = &mut*next;
            let next_stack_end = Thread::get_top_of_stack(&next.kernel_stack);

            unsafe {
                write_cr3(next.page_table);
            }

            set_current_thread(next);

            thread_switch(
                &mut current.stack_ptr, 
                next.stack_ptr, 
                next_stack_end as usize, 
                next.page_table as *const PageTable as usize);
        }
    }

    /// Get the ID of the thread.
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Get the ID of the thread.
    pub fn get_process_id(&self) -> usize {
        self.process_id
    }

    /// Prepare the stack of a newly created thread in a way that it can be used
    /// to return to the 'kickoff' function with the thread itself as parameter.
    /// The prepared stack is used in 'thread_start' to start the first thread.
    /// Other threads are started by 'thread_switch' with the prepared stack.
    fn prepare_kernel_stack(&mut self) {
        let kickoff = Thread::kickoff_kernel_thread as u64;
        let thread = ptr::from_mut(self) as u64;
        let length = self.kernel_stack.len();
        let kernel_stack_top = Self::get_top_of_stack(&self.kernel_stack);

        self.kernel_stack[length - 1] = 0x131155; // Dummy return address
        self.kernel_stack[length - 2] = kickoff; // Address of 'kickoff'
        self.kernel_stack[length - 3] = 0; // r8
        self.kernel_stack[length - 4] = 0; // r9
        self.kernel_stack[length - 5] = 0; // r10
        self.kernel_stack[length - 6] = 0; // r11
        self.kernel_stack[length - 7] = 0; // r12
        self.kernel_stack[length - 8] = 0; // r13
        self.kernel_stack[length - 9] = 0; // r14
        self.kernel_stack[length - 10] = 0; // r15
        self.kernel_stack[length - 11] = 0; // rax
        self.kernel_stack[length - 12] = 0; // rbx
        self.kernel_stack[length - 13] = 0; // rcx
        self.kernel_stack[length - 14] = 0; // rdx
        self.kernel_stack[length - 15] = 0; // rsi
        self.kernel_stack[length - 16] = thread; // rdi -> First parameter for 'kickoff'
        self.kernel_stack[length - 17] = 0; // rbp
        self.kernel_stack[length - 18] = 0x2; // rflags (IE = 0); interrupts disabled

        self.stack_ptr = kernel_stack_top as usize - (STACK_ENTRY_SIZE * 18);
    }

    /// Switch this thread from Ring 0 to Ring 3.
    /// For this, the kernel stack is prepared in a way that an 'iretq' instruction
    /// switches to user mode (Ring 3) and the user stack is used. If this function works correctly,
    /// the thread continues in user mode in the function 'kickoff_user_thread'.
    fn switch_to_usermode(&mut self) {

        // Segment Register Selector Code & Data
        // Selector = index(GDT) * size in byte | RPL
        const CS: u64 = 4*8 | 3; 
        const SS: u64 = 5*8 | 3;

        // RFLAGS
        // Interrupt Flag | Reserved (Always 1)
        const RFLAGS: u64 = 0b10_0000_0010; 

        // Usermode start address
        let rip = USER_CODE_VIRT_START as u64;

        // User stack top
        let rsp = USER_STACK_VIRT_END  as u64;

        // Self Thread stack:
        let rdi = self as *const Thread as u64;
        
        // Interrupt stack frame Layout expected by thread_user_start.
        let user_stack_top = USER_STACK_VIRT_END  as *mut u64;
        let mut stack_frame = unsafe { user_stack_top.offset(-6)};

        unsafe{  
            *stack_frame = rdi; // Self stack
            *stack_frame.add(1) = rip; // kickoff_user_thread
            *stack_frame.add(2) = CS;
            *stack_frame.add(3) = RFLAGS;
            *stack_frame.add(4) = rsp; // user stack
            *stack_frame.add(5) = SS;
        }

        unsafe {thread_user_start(stack_frame as usize)};

        // thread_user_start will not be exited 

    }

    /// Called indirectly by using the prepared stack in 'thread_start' and 'thread_switch'.
    fn kickoff_kernel_thread(&mut self) {
        // Set TSS rsp0 to the top of the kernel stack of this thread
        unsafe {
            let rsp0 = Self::get_top_of_stack(&self.kernel_stack);
            _tss_set_rsp0(rsp0 as usize);
        }

        if self.is_kernel_thread {
            cpu::enable_int(); // interrupts are disabled during thread start
            ((*self).entry)();
        } else {
            self.switch_to_usermode();
        }

        get_scheduler().exit();
    }

    /// Called indirectly by using the prepared stack in 'switch_to_usermode'.
    /// At this point, the thread is in user mode (Ring 3) and its entry function is called.
    fn kickoff_user_thread(&self) {

        (self.entry)();

        usr_thread_exit();
    }

    /// Get a pointer to the top of the given stack.
    fn get_top_of_stack(stack: &Vec<u64>) -> *const u64 {
        unsafe {
            ptr::from_ref(&stack[stack.len() - 1]).offset(1)
        }
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T{}", self.id)
    }
}