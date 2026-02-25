/*
 * Module: user_api
 *
 * Description: All system calls available to user programs are defined in this module.
 *
 * Author: Stefan Lankes, RWTH Aachen University
 *         Licensed under the Apache License, Version 2.0 or MIT license, at your option.
 *
 *         Michael Schoettner, Heinrich Heine University Duesseldorf, 14.09.2023
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 15.10.2025
 */

use core::arch::asm;

/// System call numbers available to user programs.
#[repr(u64)]
pub enum SyscallFunction {
    HelloWorld,
    ThreadYield,
    ThreadExit,
    GetThreadId,
    GetProcessId,
    DumpProcessVMAs,
    GetSystemTime,
    Print,
    GetChar,
    GetKeyQueue,
    MapHeap,
    NumSyscalls // Last entry to count number of syscalls
}

/// Test system call printing "Hello, World!" to the serial console.
pub fn usr_hello_world() {
    syscall0(SyscallFunction::HelloWorld);
}

/// Passes the CPU to the next thread
pub fn usr_thread_yield() {
    syscall0(SyscallFunction::ThreadYield);
}

/// Terminates the current thread
pub fn usr_thread_exit() {
    syscall0(SyscallFunction::ThreadExit);
}

/// Returns the ID of the current thread
pub fn usr_thread_get_id() -> usize {
    let ret = syscall0(SyscallFunction::GetThreadId);
    ret as usize
}

/// Returns the ID of the current process
pub fn usr_process_get_id() -> usize {
    let ret = syscall0(SyscallFunction::GetProcessId);
    ret as usize
}

/// Returns the ID of the current process
pub fn usr_dump_vmas() {
    syscall0(SyscallFunction::DumpProcessVMAs);
}

/// Returns the system time in ms 
pub fn usr_get_system_time() -> usize {
    let ret = syscall0(SyscallFunction::GetSystemTime);
    ret as usize
}

/// Prints a string on the current pointer position on th CGA-Screen
pub fn usr_print(msg: &str) {
    let ptr = msg.as_ptr();
    let len = msg.len();
    syscall2(SyscallFunction::Print, ptr as u64, len as u64);
}

/// Returns the next key from the keyboard buffer.
pub fn usr_get_char() -> char {
    let ret = syscall0(SyscallFunction::GetChar);
    (ret as u8) as char
}

/// Returns the next key from the keyboard buffer.
pub fn usr_get_key_queue(buf: &mut [char]) -> usize {
    syscall2(
        SyscallFunction::GetKeyQueue,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    ) as usize
}

/// Maps the user heap within the given range.
pub fn usr_map_heap(user_heap_start: u64, user_heap_size: usize) {
    syscall2(SyscallFunction::MapHeap, user_heap_start, user_heap_size as u64);
}

/// Perform a system call with 0 arguments.
#[inline(always)]
pub fn syscall0(syscall: SyscallFunction) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            options(preserves_flags, nostack)
        );
    }
    ret
}

/// Perform a system call with 1 argument.
#[inline(always)]
pub fn syscall1(syscall: SyscallFunction, arg1: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            in("rdi") arg1,
            options(preserves_flags, nostack)
        );
    }
    ret
}

/// Perform a system call with 2 arguments.
#[inline(always)]
pub fn syscall2(syscall: SyscallFunction, arg1: u64, arg2: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            options(preserves_flags, nostack)
        );
    }
    ret
}

/// Perform a system call with 3 arguments.
#[inline(always)]
pub fn syscall3(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            options(preserves_flags, nostack)
        );
    }
    ret
}

/// Perform a system call with 4 arguments.
#[inline(always)]
pub fn syscall4(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("rcx") arg4,
            options(preserves_flags, nostack)
        );
    }
    ret
}

/// Perform a system call with 5 arguments.
#[inline(always)]
pub fn syscall5(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("rcx") arg4,
            in("r8") arg5,
            options(preserves_flags, nostack)
        );
    }
    ret
}
