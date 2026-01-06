use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use usrlib::user_api::{usr_get_char, usr_get_system_time, usr_hello_world, usr_print, usr_thread_get_id};
use crate::kernel::threads::scheduler::get_scheduler;
use crate::kernel::threads::thread::Thread;

pub fn syscall_test() {
    let thread = Thread::new_user_thread(syscall_test_thread,0);
    let scheduler = get_scheduler();
    scheduler.ready(thread);
    scheduler.schedule();
}

fn syscall_test_thread() {
    usr_hello_world();

    print_id();
    

    let mut user_input: Vec<char> = Vec::new();
    loop {
        let last_char = usr_get_char();
        
        if last_char == '\r' {
            print_user_input(&user_input);
            print_system_time();
            user_input.clear();
        } else {
            usr_print(&last_char.to_string());
            user_input.push(last_char);
        }
    }
}

fn print_user_input(user_input: &Vec<char>) {
    let user_input_str: String = user_input.iter().collect();
    let text = format!("\nYou typed: \'{}\'\n", user_input_str);
    usr_print(&text);
}

fn print_system_time() {
    let time_in_s = usr_get_system_time()/1000;
    let text = format!("System time: {}s\n\n", time_in_s);
    usr_print(&text);
}
    
fn print_id() {
    let id_text = format!("Thread ID: {}\n\n", usr_thread_get_id());
    usr_print(&id_text);
}
