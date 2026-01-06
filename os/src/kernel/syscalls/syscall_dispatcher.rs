/*
 * Module: syscall_dispatcher
 *
 * Description: All system calls are routed here via the IDT syscall handler (interrupt 0x80).
 *              The system call number is passed in the rax register and is used as index into
    *              the syscall table to call the corresponding function.
 *
 * Author: Stefan Lankes, RWTH Aachen University
 *         Licensed under the Apache License, Version 2.0 or MIT license, at your option.
 *
 *         Michael Schoettner, Heinrich Heine University Duesseldorf, 23.10.2024
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 15.10.2025
 */

use core::arch::{naked_asm};
use crate::kernel::syscalls::functions::hello::sys_hello_world;
use crate::kernel::syscalls::functions::get_char::sys_get_char;
use crate::kernel::syscalls::functions::get_system_time::sys_get_system_time;
use crate::kernel::syscalls::functions::print::sys_print;
use crate::kernel::syscalls::functions::process_get_id::sys_process_get_id;
use crate::kernel::syscalls::functions::thread_exit::sys_thread_exit;
use crate::kernel::syscalls::functions::thread_get_id::sys_thread_get_id;
use crate::kernel::syscalls::functions::thread_yield::sys_thread_yield;
use usrlib::user_api::SyscallFunction;

/// Global syscall function table.
static SYSCALL_TABLE: SyscallFunctionTable = SyscallFunctionTable::new();

/// Struct to hold the syscall function pointers.
#[repr(align(64))]
#[repr(C)]
struct SyscallFunctionTable {
    table: [*const u64; SyscallFunction::NumSyscalls as usize],
}

impl SyscallFunctionTable {
    pub const fn new() -> SyscallFunctionTable {
        SyscallFunctionTable {
            table: [
                sys_hello_world as *const u64,
                sys_thread_yield as *const u64,
                sys_thread_exit as *const u64,
                sys_thread_get_id as *const u64,
                sys_process_get_id as *const u64,
                sys_get_system_time as *const u64,
                sys_print as *const u64,
                sys_get_char as *const u64
            ],
        }
    }
}

unsafe impl Send for SyscallFunctionTable {}
unsafe impl Sync for SyscallFunctionTable {}

/// System call dispatcher.
/// This function is called from the IDT syscall handler (interrupt 0x80).
/// The syscall number is passed in the rax register and is used as index into
/// the syscall table to call the corresponding function.
/// All registers (except rax) are saved and restored.
/// The syscall function's return value is passed back in rax.
#[unsafe(naked)]
pub extern "C" fn syscall_disp() {
    naked_asm!(
        // Save all registers (except rax, which contains the syscall number)
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Call syscall handler (or syscall_abort for an invalid syscall number)
        "cmp rax, {NUM_SYSCALLS}",
        "jge syscall_abort",
        "call [{SYSCALL_TABLE} + rax * 8]",

        // Restore all registers (except rax)
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",


        // Return from interrupt
        "iretq",

        NUM_SYSCALLS = const SyscallFunction::NumSyscalls as usize,
        SYSCALL_TABLE = sym SYSCALL_TABLE
    )
}

/// This function is called if an invalid syscall number is passed.
/// It panics and halts the system.
#[unsafe(no_mangle)]
extern "C" fn syscall_abort() {
    panic!("Invalid syscall number");
}
