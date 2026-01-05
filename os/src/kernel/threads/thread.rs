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
use crate::consts::{PAGE_SIZE, STACK_ENTRY_SIZE, STACK_SIZE, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
use crate::kernel::paging::frames::FRAME_ALLOCATOR;
use crate::kernel::{allocator, cpu, multiboot};
use crate::kernel::paging::pages::{self, PageFlags, PageTable, map_user_app, map_user_stack, write_cr3};
use usrlib::user_api::usr_thread_exit;
use crate::kernel::threads::scheduler::get_scheduler;

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
    user_stack: Vec<u64>,
    stack_ptr: usize, // Pointer on the stack to the saved context
    entry: fn(),
    page_table: &'static mut PageTable,
}

impl Thread {
    /// Create a new thread with the given entry function.
    pub fn new_kernel_thread(entry: fn()) -> Box<Thread> {
        // Allocate memory for the kernel stack and initialize it to zero
        let mut kernel_stack = Vec::<u64>::with_capacity(STACK_SIZE / 8);
        for _ in 0..kernel_stack.capacity() {
            kernel_stack.push(0);
        }

        // Allocate page table
        let page_table = pages::init_kernel_tables();

        // Allocate memory for the user stack and initialize it to zero
        let user_stack_addr =  unsafe { map_user_stack(page_table) }  as *mut u64;
        let mut user_stack = unsafe { 
            Vec::from_raw_parts(
            user_stack_addr, //
            STACK_SIZE/8, 
            STACK_SIZE/8) 
        };

        // Set the stack pointer to the top of the stack
        let stack_ptr = ptr::from_ref(&kernel_stack[kernel_stack.capacity() - 1]) as usize;

        // Create a new thread object
        let mut thread = Box::new(
            Thread { id: next_id(), is_kernel_thread: true, kernel_stack, user_stack, stack_ptr, entry, page_table }
        );

        // Prepare the stack for the thread so it can be started via `thread_start()`
        thread.prepare_kernel_stack();
        thread
    }

    pub fn new_user_thread(entry: fn()) -> Box<Thread> {
        // Allocate memory for the kernel stack and initialize it to zero
        let mut kernel_stack = Vec::<u64>::with_capacity(STACK_SIZE / 8);
        for _ in 0..kernel_stack.capacity() {
            kernel_stack.push(0);
        }

        // Allocate page table
        let page_table = pages::init_kernel_tables();

        // Allocate memory for the user stack and initialize it to zero
        let user_stack_addr =  unsafe { map_user_stack(page_table) } as *mut u64;
        let mut user_stack = unsafe { 
            Vec::from_raw_parts(
            user_stack_addr,
            STACK_SIZE/8, 
            STACK_SIZE/8) 
        };

        // Map user app to memory
        let mut entry_function: fn() = entry;
        let archive = multiboot::MULTIBOOT_INFO.get().expect("No MULTIBOOT_INFO").get_initrd_archive().expect("No TAR-Archive for the user app was found");
        for entry in archive.entries() {
            let num_pages = (entry.size() + PAGE_SIZE - 1) / PAGE_SIZE;
            let filename = entry.filename();
            let str :&str = filename.as_str().unwrap();
            kprintln!("TAR File found: '{}'",str);
            
            // Save data to physical address
            let phys_addr = unsafe { FRAME_ALLOCATOR.lock().alloc_block(num_pages).expect("Could not allocate physical memory for user app.") };
            let app_data = entry.data();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    app_data.as_ptr(),
                    phys_addr.as_mut_ptr::<u8>(),
                    app_data.len(),
                );
            }

            // Map user app
            let virt_addr = unsafe { map_user_app(page_table, num_pages) };
            entry_function = unsafe { core::mem::transmute(virt_addr) };
        }


        // Set the stack pointer to the top of the stack
        let stack_ptr = ptr::from_ref(&user_stack[user_stack.capacity() - 1]) as usize;

        // Create a new thread object
        let mut thread = Box::new(
            Thread { id: next_id(), is_kernel_thread: false, kernel_stack, user_stack, stack_ptr, entry : entry_function, page_table }
        );

        // Prepare the stack for the thread so it can be started via `thread_start()`
        thread.prepare_kernel_stack();

        thread
    }

    /// Start the thread.
    /// This function is only once by the scheduler.
    /// The scheduler does further thread switching via `switch()`.
    pub fn start(&mut self) {

        // Test (Manuel page walk thru pml4). TODO: Remove if Page Faults are gone
        // let addr =  0x4000_000F_FFF8 as usize; // currently not working address
        // // let addr =  USER_STACK_VIRT_END - 0xFFF as usize; // Lower Address that should work
        // let pml4e = self.page_table.entries[(addr >> 39 & 0x1FF) as usize];
        // assert!(pml4e.get_flags().contains(PageFlags::PRESENT));

        // let pdpt = unsafe { pml4e.get_addr().as_mut_ptr::<PageTable>().as_mut().unwrap() };
        // let pdpte = pdpt.entries[(addr >> 30 & 0x1FF) as usize];
        // assert!(pdpte.get_flags().contains(PageFlags::PRESENT));

        // let pd = unsafe { pdpte.get_addr().as_mut_ptr::<PageTable>().as_mut().unwrap() };
        // let pde = pd.entries[(addr >> 21 & 0x1FF) as usize];
        // assert!(pde.get_flags().contains(PageFlags::PRESENT));

        // let pt = unsafe { pde.get_addr().as_mut_ptr::<PageTable>().as_mut().unwrap() };
        // let pte = pt.entries[(addr >> 12 & 0x1FF) as usize];
        // assert!(pte.get_flags().contains(PageFlags::PRESENT), "pte.get_flags() was not PRESENT at {}", pte.get_addr().raw());

        unsafe {
            write_cr3(self.page_table);
        }

        unsafe {
            thread_start(self.stack_ptr, self.page_table as *const PageTable as usize);
        }
    }

    /// Switch from the `current` thread to the `next` thread.
    /// This function is called by the scheduler to switch between threads.
    pub unsafe fn switch(current: *mut Thread, next: *mut Thread) {
        unsafe {
            let current = &mut *current;
            let next = &*next;
            let next_stack_end = Thread::get_top_of_stack(&next.kernel_stack);

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

        // Usermode start address, should be virt_addr of 
        let rip = Thread::kickoff_user_thread as u64;

        // User stack top
        let rsp = Thread::get_top_of_stack(&self.user_stack) as u64;

        // Self Thread stack:
        let rdi = self as *const Thread as u64;
        
        // Interrupt stack frame Layout expected by thread_user_start.
        let mut stack_frame: [u64; 6] = [0; 6];
        stack_frame[0] = rdi; // Self stack
        stack_frame[1] = rip; // kickoff_user_thread
        stack_frame[2] = CS;
        stack_frame[3] = RFLAGS;
        stack_frame[4] = rsp; // user stack
        stack_frame[5] = SS;

        unsafe {thread_user_start(stack_frame.as_ptr() as usize)};

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