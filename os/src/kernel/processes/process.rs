use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::AtomicUsize;
use crate::library::mutex::Mutex;

static PROCESSES: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct Process {
    id: usize,
    name: String
}

impl Process {
    pub fn new(name: &str) -> Self {
        let pid = NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        Process { id: pid, name: String::from(name) }
    }
}

pub fn add_process(process: Process) {
    PROCESSES.lock().insert(process.id, process);
}

pub fn remove_process(process_id: usize) {
    PROCESSES.lock().remove(&process_id);
}

pub fn get_app_name(process_id: usize) -> Option<String> {
    let processes = PROCESSES.lock();
    let process = processes.get(&process_id);
    if process.is_none() {
        return None;
    }
    return Some(process.unwrap().name.clone());

}
